use std::{collections::HashMap, sync::Arc};

use super::{
    generate_gltf::generate_high_lod,
    session::{Mailbox, UiMessage},
};
use crate::{
    http_handler::{download_item, download_mesh, download_object},
    initialize::create_sub_share_dir,
};
use actix::{Addr, AsyncContext, Handler, Message, WrapFuture};
use glam::Vec3;
use log::error;
use metaverse_messages::{
    capabilities::{item::Item, scene::SceneGroup},
    ui::{
        mesh_update::{MeshType, MeshUpdate},
        ui_events::UiEventTypes,
    },
    utils::{item_metadata::ItemMetadata, object_types::ObjectType},
};
use std::sync::Mutex;
use uuid::Uuid;

#[cfg(feature = "agent")]
#[derive(Debug, Message, Clone)]
#[rtype(result = "()")]
/// This contains information about the agent appearances as they come into the scene.
/// this is used to collect agent items, and trigger generation of the GLTF once their assets have
/// been loaded.
pub struct AgentAppearance {
    /// The UUID of the agent
    pub agent_id: Uuid,
    /// How many items are being requested. When the appearance is fully loaded, the outfit_size
    /// and outfit_items will be of equal length.
    pub outfit_size: usize,
    /// The items in the outfit. Contains mesh and other data that will be used to construct the
    /// gltf file.
    pub outfit_items: Vec<OutfitObject>,
}

#[cfg(feature = "agent")]
#[derive(Debug, Message, Clone)]
#[rtype(result = "()")]
/// The items that can be stored in an outfit. It can contain generic Items, or SceneGroups, which
/// contain mesh data. 
pub enum OutfitObject {
    /// A generic item 
    Item(Item),
    /// A SceneGroup, containing mesh and render data
    SceneGroup(SceneGroup),
}

#[derive(Debug, Message)]
#[rtype(result = "()")]
/// Retrieve agent data from a capability url  
pub struct DownloadAgentAsset {
    /// The url of the capability url to retrieve data from
    pub url: String,
    /// The metadata of the item to download
    pub item: ItemMetadata,
    /// The agent ID of the avatar
    pub agent_id: Uuid,
    /// The location of the agent in space
    pub position: Vec3,
}

impl Handler<DownloadAgentAsset> for Mailbox {
    type Result = ();
    fn handle(&mut self, msg: DownloadAgentAsset, ctx: &mut Self::Context) -> Self::Result {
        if let Some(session) = self.session.as_mut() {
            let agent_list = session.agent_list.clone();
            let address = ctx.address().clone();
            ctx.spawn(
                async move {
                    match msg.item.item_type {
                        ObjectType::Object => match download_object(msg.item, &msg.url).await {
                            Ok(mut scene_group) => {
                                for scene in &scene_group.parts {
                                    let mut mesh_metadata = scene.item_metadata.clone();
                                    mesh_metadata.item_type = ObjectType::Mesh;
                                    mesh_metadata.asset_id = scene.sculpt.texture;
                                    match download_mesh(mesh_metadata, &msg.url).await {
                                        Ok(mut mesh) => {
                                            mesh.position = Some(msg.position);
                                            if let Some(ref mut mesh_list) = scene_group.meshes {
                                                mesh_list.push(mesh)
                                            } else {
                                                scene_group.meshes = Some(vec![mesh])
                                            }
                                        }
                                        Err(e) => {
                                            error!("{:?}", e);
                                        }
                                    };
                                }
                                add_item_to_agent_list(
                                    agent_list,
                                    msg.agent_id,
                                    OutfitObject::SceneGroup(scene_group),
                                    address,
                                );
                            }
                            Err(e) => {
                                error!("{:?}", e);
                            }
                        },
                        ObjectType::Link => {}
                        _ => match download_item(msg.item, &msg.url).await {
                            Ok(item) => {
                                add_item_to_agent_list(
                                    agent_list,
                                    msg.agent_id,
                                    OutfitObject::Item(item),
                                    address,
                                );
                            }
                            Err(e) => {
                                error!("{:?}", e);
                            }
                        },
                    }
                }
                .into_actor(self),
            );
        }
    }
}

fn add_item_to_agent_list(
    agent_list: Arc<Mutex<HashMap<Uuid, AgentAppearance>>>,
    agent_id: Uuid,
    item: OutfitObject,
    address: Addr<Mailbox>,
) {
    if let Some(agent) = agent_list.lock().unwrap().get_mut(&agent_id) {
        agent.outfit_items.push(item);
        // if all of the items have loaded in, trigger a render
        if agent.outfit_items.len() == agent.outfit_size {
            address.do_send(agent.clone());
        }
    }
}

impl Handler<AgentAppearance> for Mailbox {
    type Result = ();
    fn handle(&mut self, msg: AgentAppearance, ctx: &mut Self::Context) -> Self::Result {
        for item in msg.outfit_items {
            match item {
                OutfitObject::SceneGroup(scene_group) => {
                    if let Ok(agent_dir) = create_sub_share_dir("agent") {
                        if let Some(meshes) = scene_group.meshes {
                            for mesh in meshes {
                                match generate_high_lod(
                                    &mesh,
                                    agent_dir.clone(),
                                    msg.agent_id.to_string(),
                                ) {
                                    Ok(path) => ctx.address().do_send(UiMessage::new(
                                        UiEventTypes::MeshUpdate,
                                        MeshUpdate {
                                            position: mesh.position.unwrap(),
                                            path,
                                            mesh_type: MeshType::Avatar,
                                            id: Some(msg.agent_id),
                                        }
                                        .to_bytes(),
                                    )),
                                    Err(e) => {
                                        error!("Failed to generate GLTF {:?}", e)
                                    }
                                };
                            }
                        }
                    }
                }
                _ => {}
            }
        }
    }
}

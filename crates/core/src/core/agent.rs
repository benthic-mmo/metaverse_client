use super::session::{Mailbox, UiMessage};
use crate::http_handler::{download_item, download_mesh, download_object};
use crate::initialize::create_sub_share_dir;
use actix::{Addr, AsyncContext, Handler, Message, WrapFuture};
use glam::{Vec3, Vec4};
use log::error;
use metaverse_agent::skeleton::create_skeleton;
use metaverse_agent::{
    avatar::{Avatar, OutfitObject, RiggedObject},
    generate_gltf::generate_baked_avatar,
};
use metaverse_messages::capabilities::scene::SceneGroup;
use metaverse_messages::{
    ui::{
        mesh_update::{MeshType, MeshUpdate},
        ui_events::UiEventTypes,
    },
    utils::{item_metadata::ItemMetadata, object_types::ObjectType},
};
use std::{collections::HashMap, sync::Arc};
use std::{path::PathBuf, sync::Mutex};
use uuid::Uuid;

#[cfg(feature = "agent")]
#[derive(Debug, Message, Clone)]
#[rtype(result = "()")]
pub struct Agent {
    pub avatar: Avatar,
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

async fn download_scenegroup_mesh(scene_group: &mut SceneGroup, url: &str) {
    for scene in &mut scene_group.parts {
        let mut mesh_metadata = scene.item_metadata.clone();
        mesh_metadata.item_type = ObjectType::Mesh;
        mesh_metadata.asset_id = scene.sculpt.texture;
        match download_mesh(mesh_metadata, url).await {
            Ok(mut mesh) => {
                // apply the skin bind shape matrix to the retrieved
                // vertices
                let vertices_transformed: Vec<Vec3> = mesh
                    .high_level_of_detail
                    .vertices
                    .iter()
                    .map(|v| {
                        let v4 = mesh.skin.bind_shape_matrix * Vec4::new(v.x, v.y, v.z, 1.0);
                        Vec3::new(v4.x, v4.y, v4.z)
                    })
                    .collect();
                mesh.high_level_of_detail.vertices = vertices_transformed;
                scene.sculpt.mesh = Some(mesh);
            }
            Err(e) => {
                error!("{:?}", e);
            }
        };
    }
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
                                download_scenegroup_mesh(&mut scene_group, &msg.url).await;

                                let skeleton = create_skeleton(
                                    scene_group.parts[0].clone(),
                                    agent_list.clone(),
                                    msg.agent_id,
                                )
                                .unwrap();
                                add_item_to_agent_list(
                                    agent_list,
                                    msg.agent_id,
                                    OutfitObject::RiggedObject(RiggedObject {
                                        scene_group,
                                        skeleton,
                                    }),
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
    agent_list: Arc<Mutex<HashMap<Uuid, Avatar>>>,
    agent_id: Uuid,
    item: OutfitObject,
    address: Addr<Mailbox>,
) {
    if let Some(agent) = agent_list.lock().unwrap().get_mut(&agent_id) {
        agent.outfit_items.push(item);
        // if all of the items have loaded in, trigger a render
        if agent.outfit_items.len() == agent.outfit_size {
            address.do_send(Agent {
                avatar: agent.clone(),
            });
        }
    }
}

impl Handler<Agent> for Mailbox {
    type Result = ();
    fn handle(&mut self, mut msg: Agent, ctx: &mut Self::Context) -> Self::Result {
        // bake all of the objects in the outfit onto the same skeleton for rendering
        let path = create_sub_share_dir("agent")
            .ok()
            .and_then(|agent_dir| {
                let agent_path = msg.avatar.agent_id;
                generate_baked_avatar(
                    msg.avatar.outfit_items.clone(),
                    msg.avatar.skeleton,
                    agent_dir.with_file_name(format!("{:?}_combined_high.glb", agent_path)),
                )
                .ok()
            })
            .unwrap();
        msg.avatar.path = Some(path);
        ctx.address().do_send(UiMessage::new(
            UiEventTypes::MeshUpdate,
            MeshUpdate {
                position: msg.avatar.position,
                path: msg.avatar.path.unwrap(),
                mesh_type: MeshType::Avatar,
                id: Some(msg.avatar.agent_id),
            }
            .to_bytes(),
        ));
    }
}

use super::session::{Mailbox, UiMessage};
use crate::http_handler::{download_item, download_mesh, download_object};
use crate::initialize::create_sub_agent_dir;
use actix::{Addr, AsyncContext, Handler, Message, WrapFuture};
use glam::{Vec3, Vec4};
use log::{error, warn};
use metaverse_agent::avatar::{Avatar, OutfitObject, RiggedObject};
use metaverse_agent::skeleton::{create_skeleton, update_global_avatar_skeleton};
use metaverse_gltf::skinned_mesh::generate_skinned_mesh;
use metaverse_messages::capabilities::scene::SceneGroup;
use metaverse_messages::{
    ui::{
        mesh_update::{MeshType, MeshUpdate},
        ui_events::UiEventTypes,
    },
    utils::{item_metadata::ItemMetadata, object_types::ObjectType},
};
use serde::Serialize;
use std::fs::File;
use std::io::{self, Write};
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

                                let skeleton =
                                    create_skeleton(scene_group.parts[0].clone()).unwrap();

                                // TODO: DO NOT LEAVE THIS AS NAME PERMANENTLY. FIGURE OUT WHY THE
                                // SCENEOBJECT'S METADATA IS NOT POPULATING
                                let json_path = write_json(
                                    &scene_group,
                                    msg.agent_id,
                                    scene_group.parts[0].name.clone(),
                                )
                                .unwrap();

                                add_item_to_agent_list(
                                    agent_list,
                                    msg.agent_id,
                                    OutfitObject::RiggedObject(RiggedObject {
                                        scene_group,
                                        skeleton,
                                        json_path,
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
        // update the avatar skeleton with the calculated object skeleton
        match &item {
            OutfitObject::RiggedObject(object) => {
                update_global_avatar_skeleton(agent, &object.skeleton);
            }
            _ => {}
        }
        agent.outfit_items.push(item);
        // if all of the items have loaded in, trigger a render
        if agent.outfit_items.len() == agent.outfit_size {
            address.do_send(Agent {
                avatar: agent.clone(),
            });
        }
    }
}

fn write_json<T: Serialize>(data: &T, agent_id: Uuid, filename: String) -> io::Result<PathBuf> {
    match create_sub_agent_dir(&agent_id.to_string()) {
        Ok(mut agent_dir) => match serde_json::to_string(&data) {
            Ok(json) => {
                agent_dir.push(format!("{:?}.json", filename));
                let mut file = File::create(&agent_dir).unwrap();
                file.write_all(json.as_bytes()).unwrap();
                return Ok(agent_dir);
            }
            Err(e) => {
                error!("Failed to serialize scene group {:?}, {:?}", filename, e);
                return Err(io::Error::new(io::ErrorKind::Other, e));
            }
        },
        Err(e) => {
            error!(
                "Failed to create agent dir for {:?}. Unable to cache downloaded items.",
                e
            );
            return Err(io::Error::new(io::ErrorKind::Other, e));
        }
    };
}

impl Handler<Agent> for Mailbox {
    type Result = ();
    fn handle(&mut self, mut msg: Agent, ctx: &mut Self::Context) -> Self::Result {
        // Base "agent" directory (e.g. /share/agent)
        let agent_dir = match create_sub_agent_dir(&msg.avatar.agent_id.to_string()) {
            Ok(dir) => dir,
            Err(e) => {
                warn!("Failed to create base agent directory: {:?}", e);
                return;
            }
        };

        let skeleton_path = write_json(
            &msg.avatar.skeleton,
            msg.avatar.agent_id,
            format!("{:?}_skeleton", msg.avatar.agent_id),
        )
        .unwrap();

        // TODO: This needs major reworking
        let glb_path = agent_dir.join(format!("{:?}_combined_high.glb", msg.avatar.agent_id));
        let paths: Vec<_> = msg
            .avatar
            .outfit_items
            .iter()
            .filter_map(|obj| match obj {
                OutfitObject::RiggedObject(r) => Some(r.json_path.clone()), // extract path
                _ => None,                                                  // skip other variants
            })
            .collect();
        generate_skinned_mesh(paths, skeleton_path, glb_path.clone());

        // Update avatar state and notify UI
        msg.avatar.path = Some(glb_path.clone());
        ctx.address().do_send(UiMessage::new(
            UiEventTypes::MeshUpdate,
            MeshUpdate {
                position: msg.avatar.position,
                path: glb_path,
                mesh_type: MeshType::Avatar,
                id: Some(msg.avatar.agent_id),
            }
            .to_bytes(),
        ));
    }
}

use super::session::Mailbox;
use crate::initialize::create_sub_agent_dir;
use crate::transport::http_handler::{download_item, download_mesh, download_object};
use actix::{Addr, AsyncContext, Handler, Message, WrapFuture};
use glam::{Vec3, Vec4};
use log::{error, info, warn};
use metaverse_agent::avatar::{Avatar, MeshObject, OutfitObject};
use metaverse_agent::skeleton::{create_skeleton, update_global_avatar_skeleton};
use metaverse_mesh::skinned_mesh::generate_skinned_mesh;
use metaverse_messages::http::scene::SceneGroup;
use metaverse_messages::packet::message::UIMessage;
use metaverse_messages::utils::render_data::{AvatarObject, RenderObject, SkinData};
use metaverse_messages::utils::skeleton::Skeleton;
use metaverse_messages::{
    ui::mesh_update::{MeshType, MeshUpdate},
    utils::{item_metadata::ItemMetadata, object_types::ObjectType},
};
use serde::Serialize;
use std::fs::{self, File};
use std::io::{self, Write};
use std::{collections::HashMap, sync::Arc};
use std::{path::PathBuf, sync::Mutex};
use uuid::Uuid;

#[cfg(feature = "agent")]
#[derive(Debug, Message, Clone)]
#[rtype(result = "()")]
/// A wrapper struct for the Avatar object, to allow sending it using actix
pub struct Agent {
    /// The avatar object that contains the avatar's information
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

/// Handle downloading assets.
/// this must be done in two parts.
impl Handler<DownloadAgentAsset> for Mailbox {
    type Result = ();
    fn handle(&mut self, msg: DownloadAgentAsset, ctx: &mut Self::Context) -> Self::Result {
        if let Some(session) = self.session.as_mut() {
            let agent_list = session.agent_list.clone();
            let address = ctx.address().clone();
            // do the downloading asyncronously.
            ctx.spawn(
                async move {
                    match msg.item.item_type {
                        // if an item's type is Object, that means it has a mesh.
                        ObjectType::Object => match download_object(msg.item, &msg.url).await {
                            Ok(scene_group) => {
                                let render_objects =
                                    match download_render_object(&scene_group, &msg.url).await {
                                        Ok(objects) => objects,
                                        Err(e) => {
                                            error!("{:?}", e);
                                            return; // or handle error differently
                                        }
                                    };
                                // write the object to disk as json, to give to the code that will
                                // generate the 3d model.
                                // TODO: DO NOT LEAVE THIS AS NAME PERMANENTLY. FIGURE OUT WHY THE
                                // SCENEOBJECT'S METADATA IS NOT POPULATING
                                let json_path_2 = write_json(
                                    &render_objects,
                                    msg.agent_id,
                                    scene_group.parts[0].name.clone(),
                                )
                                .unwrap();

                                // add the item to the global agent object.
                                add_item_to_agent_list(
                                    agent_list,
                                    msg.agent_id,
                                    OutfitObject::MeshObject(MeshObject {
                                        json_path: json_path_2,
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

/// When the mailbox receives an Agent object, that means the avatar is finished, and ready for
/// rendering.
impl Handler<Agent> for Mailbox {
    type Result = ();
    fn handle(&mut self, mut msg: Agent, ctx: &mut Self::Context) -> Self::Result {
        // create this agent's subdirectory
        let agent_dir = match create_sub_agent_dir(&msg.avatar.agent_id.to_string()) {
            Ok(dir) => dir,
            Err(e) => {
                warn!("Failed to create base agent directory: {:?}", e);
                return;
            }
        };
        let paths: Vec<_> = msg
            .avatar
            .outfit_items
            .iter()
            .filter_map(|obj| match obj {
                OutfitObject::MeshObject(r) => Some(r.json_path.clone()), // extract path
                _ => None,                                                // skip other variants
            })
            .collect();

        let agent = AvatarObject {
            objects: paths,
            global_skeleton: msg.avatar.skeleton,
        };

        let json_path =
            write_json(&agent, msg.avatar.agent_id, msg.avatar.agent_id.to_string()).unwrap();

        let glb_path = agent_dir.join(format!("{:?}_high.glb", msg.avatar.agent_id));
        // This generates a 3d model with the final skeleton applied, out of all of the paths to
        // the json scene data retrieved from the avatar.
        match generate_skinned_mesh(json_path, glb_path.clone()) {
            Ok(_) => {
                info!("Rendering avatar at: {:?}", msg.avatar.agent_id)
            }
            Err(e) => warn!("{:?}", e),
        };

        // Update avatar state and notify UI
        msg.avatar.path = Some(glb_path.clone());
        ctx.address()
            .do_send(UIMessage::new_mesh_update(MeshUpdate {
                position: msg.avatar.position,
                path: glb_path,
                mesh_type: MeshType::Avatar,
                id: Some(msg.avatar.agent_id),
            }));
    }
}

async fn download_render_object(
    scene_group: &SceneGroup,
    url: &str,
) -> Result<Vec<RenderObject>, std::io::Error> {
    let mut meshes = Vec::new();
    for scene in &scene_group.parts {
        let mut metadata = scene.item_metadata.clone();
        metadata.item_type = ObjectType::Mesh;
        metadata.asset_id = scene.sculpt.texture;
        let mesh = download_mesh(metadata, url).await?;
        // apply the skin bind shape matrix to the retrieved
        // vertices
        let vertices_transformed: Vec<Vec3> = mesh
            .high_level_of_detail
            .vertices
            .iter()
            .map(|v| {
                let v4 =
                    mesh.skin.as_ref().unwrap().bind_shape_matrix * Vec4::new(v.x, v.y, v.z, 1.0);
                Vec3::new(v4.x, v4.y, v4.z)
            })
            .collect();

        let skin = if mesh.skin.is_some() {
            // create the object specific skeleton. This contains no default
            // bone information, and only contains the skeleton data relevant
            // to this specific object.
            let skeleton = create_skeleton(
                scene.name.clone(),
                scene.sculpt.texture,
                mesh.skin.as_ref().unwrap(),
            )
            .unwrap_or_else(|e| {
                println!("Failed to create skeleton: {:?}", e);
                Skeleton::default()
            });
            Some(SkinData {
                skeleton: skeleton,
                weights: mesh.high_level_of_detail.weights.unwrap(),
                joint_names: mesh.skin.clone().unwrap().joint_names,
                inverse_bind_matrices: mesh.skin.clone().unwrap().inverse_bind_matrices,
            })
        } else {
            None
        };
        meshes.push(RenderObject {
            name: scene.name.clone(),
            id: scene.sculpt.texture,
            indices: mesh.high_level_of_detail.indices,
            vertices: vertices_transformed,
            skin: skin,
        });
    }
    Ok(meshes)
}

/// When an outfit item is fully downloaded, it should be added to the agent, and its global
/// skeleton updated.
fn add_item_to_agent_list(
    agent_list: Arc<Mutex<HashMap<Uuid, Avatar>>>,
    agent_id: Uuid,
    item: OutfitObject,
    address: Addr<Mailbox>,
) {
    if let Some(agent) = agent_list.lock().unwrap().get_mut(&agent_id) {
        // update the global avatar skeleton with the calculated object skeleton
        if let OutfitObject::MeshObject(object) = &item {
            let json_str = fs::read_to_string(&object.json_path)
                .expect(&format!("Failed to read {:?}", object.json_path));
            let parts: Vec<RenderObject> = serde_json::from_str(&json_str).unwrap();

            if let Some(skin) = &parts[0].skin {
                update_global_avatar_skeleton(agent, &skin.skeleton);
            }
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

/// When an object is retrieved in full, the data will be written in serializable json format, to
/// create a cache. The JSON will then be sent to another crate to convert it into a 3d model that
/// can be rendered.
fn write_json<T: Serialize>(data: &T, agent_id: Uuid, filename: String) -> io::Result<PathBuf> {
    match create_sub_agent_dir(&agent_id.to_string()) {
        Ok(mut agent_dir) => match serde_json::to_string(&data) {
            Ok(json) => {
                agent_dir.push(format!("{}.json", filename));
                let mut file = File::create(&agent_dir).unwrap();
                file.write_all(json.as_bytes()).unwrap();
                Ok(agent_dir)
            }
            Err(e) => {
                error!("Failed to serialize scene group {:?}, {:?}", filename, e);
                Err(io::Error::other(e))
            }
        },
        Err(e) => {
            error!(
                "Failed to create agent dir for {:?}. Unable to cache downloaded items.",
                e
            );
            Err(io::Error::other(e))
        }
    }
}

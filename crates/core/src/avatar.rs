use super::session::Mailbox;
use crate::initialize::create_sub_agent_dir;
use crate::transport::http_handler::{download_mesh, download_object, download_texture};
use actix::{AsyncContext, Handler, Message, WrapFuture};
use glam::{Vec3, Vec4};
use log::{error, info, warn};
use metaverse_agent::avatar::Avatar;
use metaverse_agent::avatar::OutfitObject;
use metaverse_agent::skeleton::{create_skeleton, update_global_avatar_skeleton};
use metaverse_inventory::agent::get_current_outfit;
use metaverse_mesh::generate::generate_skinned_mesh;
use metaverse_messages::http::capabilities::Capability;
use metaverse_messages::http::scene::SceneGroup;
use metaverse_messages::packet::message::UIMessage;
use metaverse_messages::ui::mesh_update::{MeshType, MeshUpdate};
use metaverse_messages::utils::object_types::ObjectType;
use metaverse_messages::utils::render_data::{AvatarObject, RenderObject, SkinData};
use metaverse_messages::utils::skeleton::Skeleton;
use serde::Serialize;
use std::fs::{self, File};
use std::io::{self, Write};
use std::path::PathBuf;
use std::time::Duration;
use uuid::Uuid;

#[derive(Debug, Message)]
#[rtype(result = "()")]
/// Requests data from the ViewerAsset capability endpoint
pub struct DownloadAgentAsset {
    /// the asset of the object to request
    pub asset_id: Uuid,
    /// the ID of the agent the asset belongs to
    pub agent_id: Uuid,
    /// the object type, used to retrieve from the capability endpoint
    pub item_type: ObjectType,
}

#[derive(Debug, Message)]
#[rtype(result = "()")]
/// add an object to the avatar's object list
pub struct AddObjectToAvatar {
    /// ID of the agent to add the object to
    pub agent_id: Uuid,
    /// Object to add
    pub object: OutfitObject,
}

#[derive(Debug, Message)]
#[rtype(result = "()")]
/// set the outfit size of the avatar.
/// retrieved from the current outfit folder.
/// this must be set in order to trigger a render. the avatar must know it has loaded all of its
/// clothes.
pub struct SetOutfitSize {
    /// agent ID to set the outfit size of
    pub agent_id: Uuid,
    /// size of the outfit
    pub outfit_size: usize,
}

#[derive(Debug, Message)]
#[rtype(result = "()")]
/// Finalize the avatar and generate the finished, rigged model.
pub struct FinalizeAvatar {
    /// the avatar object to generate
    pub avatar: Avatar,
}

impl Handler<SetOutfitSize> for Mailbox {
    type Result = ();
    fn handle(&mut self, msg: SetOutfitSize, _ctx: &mut Self::Context) -> Self::Result {
        if let Some(session) = self.session.as_mut()
            && let Some(agent) = session.avatars.get_mut(&msg.agent_id) {
                agent.outfit_size = msg.outfit_size;
            }
    }
}

impl Handler<AddObjectToAvatar> for Mailbox {
    type Result = ();
    fn handle(&mut self, msg: AddObjectToAvatar, ctx: &mut Self::Context) -> Self::Result {
        if let Some(session) = self.session.as_mut() {
            if let Some(avatar) = session.avatars.get_mut(&msg.agent_id) {
                match &msg.object {
                    OutfitObject::MeshObject(path) => {
                        let json_str = fs::read_to_string(&path)
                            .unwrap_or_else(|_| panic!("Failed to read {:?}", path));

                        let parts: Vec<RenderObject> = serde_json::from_str(&json_str).unwrap();

                        if let Some(skin) = &parts[0].skin {
                            update_global_avatar_skeleton(avatar, &skin.skeleton);
                        }
                    }
                    _ => {
                        //TODO: unimplemented!
                    }
                }

                avatar.items.push(msg.object);

                if avatar.items.len() == avatar.outfit_size {
                    ctx.address().do_send(FinalizeAvatar {
                        avatar: avatar.clone(),
                    });
                }
            } else {
                warn!("Agent not found for agent_id {:?}", &msg.agent_id);
            }
        }
    }
}

impl Handler<FinalizeAvatar> for Mailbox {
    type Result = ();
    fn handle(&mut self, msg: FinalizeAvatar, ctx: &mut Self::Context) -> Self::Result {
        let agent_dir = match create_sub_agent_dir(&msg.avatar.agent_id.to_string()) {
            Ok(dir) => dir,
            Err(e) => {
                warn!("Failed to create base agent directory: {:?}", e);
                return;
            }
        };
        let json_paths: Vec<PathBuf> = msg
            .avatar
            .items
            .iter()
            .filter_map(|item| {
                if let OutfitObject::MeshObject(path) = item {
                    Some(path.clone())
                } else {
                    None
                }
            })
            .collect();

        let avatar = AvatarObject {
            objects: json_paths,
            global_skeleton: msg.avatar.skeleton,
        };

        let json_path = write_json(
            &avatar,
            msg.avatar.agent_id,
            msg.avatar.agent_id.to_string(),
        )
        .unwrap();

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
        ctx.address()
            .do_send(UIMessage::new_mesh_update(MeshUpdate {
                position: msg.avatar.position,
                path: glb_path,
                mesh_type: MeshType::Avatar,
                id: Some(msg.avatar.agent_id),
            }));
    }
}

impl Handler<DownloadAgentAsset> for Mailbox {
    type Result = ();
    fn handle(&mut self, msg: DownloadAgentAsset, ctx: &mut Self::Context) -> Self::Result {
        if let Some(session) = self.session.as_mut() {
            let server_endpoint = session
                .capability_urls
                .get(&Capability::ViewerAsset)
                .unwrap()
                .to_string();
            let addr = ctx.address();
            ctx.spawn(
                async move {
                    match download_object(msg.item_type.to_string(), msg.asset_id, &server_endpoint)
                        .await
                    {
                        Ok(scene_group) => {
                            // if there is a scene group, download the renderable objects
                            let json_path = match download_render_object(
                                &scene_group,
                                msg.agent_id,
                                &server_endpoint,
                            )
                            .await
                            {
                                Ok(objects) => objects,
                                Err(e) => {
                                    error!("{:?}", e);
                                    return;
                                }
                            };
                            addr.do_send(AddObjectToAvatar {
                                object: OutfitObject::MeshObject(json_path),
                                agent_id: msg.agent_id,
                            });
                        }
                        Err(e) => {
                            error!("{:?}, {:?}", e, msg);
                        }
                    }
                }
                .into_actor(self),
            );
        }
    }
}

impl Handler<Avatar> for Mailbox {
    type Result = ();
    fn handle(&mut self, msg: Avatar, ctx: &mut Self::Context) -> Self::Result {
        if let Some(session) = self.session.as_mut() {
            let addr = ctx.address();
            if session.inventory_data.inventory_init {
                session.avatars.insert(msg.agent_id, msg);
                let agent_id = session.agent_id;
                let db_conn = self.inventory_db_connection.clone();
                ctx.spawn(
                    async move {
                        match get_current_outfit(&db_conn).await {
                            Ok(outfit) => {
                                if let Err(err) = addr
                                    .send(SetOutfitSize {
                                        agent_id,
                                        outfit_size: outfit.len(),
                                    })
                                    .await
                                {
                                    error!(
                                        "Failed to set outfit size for {:?}: {:?}",
                                        agent_id, err
                                    );
                                    return;
                                };
                                for item in outfit {
                                    match item.2 {
                                        ObjectType::Object => {
                                            addr.do_send(DownloadAgentAsset {
                                                asset_id: item.1,
                                                item_type: item.2,
                                                agent_id,
                                            });
                                        }
                                        ObjectType::Bodypart => {
                                            addr.do_send(AddObjectToAvatar {
                                                object: OutfitObject::Bodypart,
                                                agent_id,
                                            });
                                        }
                                        ObjectType::Clothing => {
                                            addr.do_send(AddObjectToAvatar {
                                                object: OutfitObject::Clothing,
                                                agent_id,
                                            });
                                        }
                                        _ => {
                                            addr.do_send(AddObjectToAvatar {
                                                object: OutfitObject::Other,
                                                agent_id,
                                            });
                                        }
                                    }
                                }
                            }
                            Err(e) => {
                                error!("Failed to retrieve inventory {:?}", e);
                            }
                        };
                    }
                    .into_actor(self),
                );
            } else {
                warn!("Inventory not yet ready. Requeueing avatar download...");
                ctx.notify_later(msg, Duration::from_secs(1));
            }
        }
    }
}

async fn download_render_object(
    scene_group: &SceneGroup,
    root_id: Uuid,
    url: &str,
) -> Result<PathBuf, std::io::Error> {
    let mut meshes = Vec::new();
    for scene in &scene_group.parts {
        let mesh = download_mesh(ObjectType::Mesh.to_string(), scene.sculpt.texture, url).await?;
        let texture = download_texture(
            ObjectType::Texture.to_string(),
            scene.shape.texture.texture_id,
            url,
        )
        .await?;
        let base_dir = create_sub_agent_dir(&root_id.to_string())?;
        let texture_path = base_dir.join(format!("{:?}.png", scene.shape.texture.texture_id));
        texture.save(&texture_path).unwrap();

        let domain = &mesh.high_level_of_detail.texture_coordinate_domain;

        let uvs: Vec<[f32; 2]> = mesh
            .high_level_of_detail
            .texture_coordinate
            .iter()
            .map(|tc| {
                // Normalize U and V from 0..65535 to 0..1
                let u_norm = tc.u as f32 / 65535.0;
                let v_norm = tc.v as f32 / 65535.0;

                // Flip V axis
                let v_flipped = 1.0 - v_norm;

                [
                    domain.min[0] + u_norm * (domain.max[0] - domain.min[0]),
                    domain.min[1] + v_flipped * (domain.max[1] - domain.min[1]),
                ]
            })
            .collect();

        if let Some(skin) = &mesh.skin {
            // Apply bind shape matrix
            let vertices: Vec<Vec3> = mesh
                .high_level_of_detail
                .vertices
                .iter()
                .map(|v| {
                    let v4 = skin.bind_shape_matrix * Vec4::new(v.x, v.y, v.z, 1.0);
                    Vec3::new(v4.x, v4.y, v4.z)
                })
                .collect();

            let skeleton = create_skeleton(scene.metadata.name.clone(), scene.sculpt.texture, skin)
                .unwrap_or_else(|e| {
                    println!("Failed to create skeleton: {:?}", e);
                    Skeleton::default()
                });

            let skin_data = SkinData {
                skeleton,
                weights: mesh
                    .high_level_of_detail
                    .weights
                    .clone()
                    .unwrap_or_default(),
                joint_names: skin.joint_names.clone(),
                inverse_bind_matrices: skin.inverse_bind_matrices.clone(),
            };

            meshes.push(RenderObject {
                name: scene.metadata.name.clone(),
                id: scene.sculpt.texture,
                indices: mesh.high_level_of_detail.indices,
                vertices,
                skin: Some(skin_data),
                texture: Some(texture_path),
                uv: Some(uvs),
            });
        } else {
            // No skin
            meshes.push(RenderObject {
                name: scene.metadata.name.clone(),
                id: scene.sculpt.texture,
                indices: mesh.high_level_of_detail.indices,
                vertices: mesh.high_level_of_detail.vertices,
                skin: None,
                texture: Some(texture_path),
                uv: Some(uvs),
            });
        }
    }
    // write the object to disk as json, to give to the code that will
    // generate the 3d model.
    let json_path = write_json(
        &meshes,
        root_id,
        format!(
            "{:?}_{}",
            scene_group.parts[0].sculpt.texture, scene_group.parts[0].metadata.name
        ),
    )
    .unwrap();

    Ok(json_path)
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

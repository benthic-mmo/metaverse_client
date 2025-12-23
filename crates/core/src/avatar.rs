use super::session::Mailbox;
use crate::initialize::create_sub_agent_dir;
use crate::session::SendUIMessage;
use crate::transport::http_handler::{download_object, download_scene_group, download_texture};
use actix::{AsyncContext, Handler, Message, WrapFuture};
use log::{error, info, warn};
use metaverse_agent::avatar::Avatar;
use metaverse_agent::avatar::OutfitObject;
use metaverse_agent::skeleton::update_global_avatar_skeleton;
use metaverse_inventory::agent::get_current_outfit;
use metaverse_mesh::generate::generate_skinned_mesh;
use metaverse_messages::http::capabilities::Capability;
use metaverse_messages::packet::message::UIMessage;
use metaverse_messages::udp::agent::avatar_appearance::AvatarAppearance;
use metaverse_messages::ui::camera_position::CameraPosition;
use metaverse_messages::ui::mesh_update::{MeshType, MeshUpdate};
use metaverse_messages::utils::object_types::ObjectType;
use metaverse_messages::utils::render_data::{AvatarObject, RenderObject};
use serde::Serialize;
use std::fs::{self, File};
use std::io::{self, Write};
use std::path::PathBuf;
use std::time::Duration;
use uuid::Uuid;

/// Requests Agent data from the ViewerAsset capability endpoint
///
/// These are requested from inventory objects, and not ObjectUpdate packets.
///
/// # Cause
/// - [`HandleNewAvatar`]
///
/// # Effects
///  - Dispatches an [`AddObjectToAvatar`] message on successful download
#[derive(Debug, Message)]
#[rtype(result = "()")]
pub struct DownloadAgentAsset {
    /// the asset of the object to request
    pub asset_id: Uuid,
    /// the ID of the agent the asset belongs to
    pub agent_id: Uuid,
    /// the object type, used to retrieve from the capability endpoint
    pub item_type: ObjectType,
}

/// Adds an object to the avatar's outfit list
///
/// This updates the in-memory struct containing each player's avatar data, and updates
/// the global avatar skeleton if the outfit contains skeleton data, and triggers a render
/// if the avatar's outfit is finished loading in.
///
/// # Cause
/// - [`AddObjectToAvatar`]
///
/// # Effects
/// - Dispatches a [`FinalizeAvatar`] message if the avatar's outfit has all items
#[derive(Debug, Message)]
#[rtype(result = "()")]
pub struct AddObjectToAvatar {
    /// ID of the agent to add the object to
    pub agent_id: Uuid,
    /// Object to add
    pub object: OutfitObject,
}

/// Message to set the outfit size of the avatar
///
/// This must be set in order to trigger a render. Without this, the avatar doesn't know when the
/// outfit is finished loading in.
///
/// # Cause
/// - [`HandleNewAvatar`]
#[derive(Debug, Message)]
#[rtype(result = "()")]
pub struct SetOutfitSize {
    /// agent ID to set the outfit size of
    pub agent_id: Uuid,
    /// size of the outfit
    pub outfit_size: usize,
}

/// Message to finalize the avatar
///
/// This is triggered when all of the avatar's objects have loaded in. This finalizes the global
/// skeleton, writes the JSON for the full baked avatar, generates the mesh from metaverse-mesh,
/// and triggers a UI update.
///
/// # Cause
/// - [`AddObjectToAvatar`]
///
/// # Effects
/// - Dispatches a [`MeshUpdate`] message to render the finalized avatar
#[derive(Debug, Message)]
#[rtype(result = "()")]
pub struct FinalizeAvatar {
    /// the avatar object to generate
    pub avatar: Avatar,
}

/// Message to spawn a new avatar
///
/// Avatars are added to the scene via ObjectUpdate packet with minimal information aside from the
/// ID. This adds those IDs to the session and begins downloading their appearances.
///
/// # Cause
/// - [`HandleObjectUpdate`]
///
/// # Effect
/// - Resends a [`HandleNewAvatar`] message if the inventory is not yet loaded
/// - If the avatar is the current player's avatar
///    - Dispatches a [`CameraPosition`] message
///    - Dispatches a [`DownloadAgentAsset`] message for each object in the outfit
///    - Dispatches a [`AddObjectToAvatar`] message for each bodypart in the outfit
#[derive(Debug, Message)]
#[rtype(result = "()")]
pub struct HandleNewAvatar {
    /// the avatar object to generate
    pub avatar: Avatar,
}

/// Message to handle an updated avatar appearance
///
/// TODO: currently a stub
///
/// # Cause
/// - Avatar Appearance packet received from UDP socket
#[derive(Debug, Message)]
#[rtype(result = "()")]
pub struct HandleNewAvatarAppearance {
    /// the avatar appearance data to handle
    pub avatar_appearance: AvatarAppearance,
}

impl Handler<HandleNewAvatarAppearance> for Mailbox {
    type Result = ();
    fn handle(
        &mut self,
        _msg: HandleNewAvatarAppearance,
        _ctx: &mut Self::Context,
    ) -> Self::Result {
        // TODO: implement this. AvatarAppearance packets change skeleton joint positions to change
        // the appearance of the avatar.
        warn!("AvatarAppearance packet received. Currently unimplemented");
    }
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
        ctx.address().do_send(SendUIMessage {
            ui_message: UIMessage::new_mesh_update(MeshUpdate {
                position: msg.avatar.position,
                path: glb_path,
                mesh_type: MeshType::Avatar,
                id: Some(msg.avatar.agent_id),
            }),
        });
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
                            let base_dir = match create_sub_agent_dir(&msg.agent_id.to_string()) {
                                Ok(base_dir) => base_dir,
                                Err(e) => {
                                    error!("failed to create base dir: {:?}", e);
                                    return;
                                }
                            };

                            // download the texture of the base object, which will have the texture
                            // for the rest of the object
                            let texture_id = scene_group.parts[0].shape.texture.texture_id;
                            let texture_path = base_dir.join(format!("{:?}.png", texture_id));
                            match download_texture(
                                ObjectType::Texture.to_string(),
                                texture_id,
                                &server_endpoint,
                                &texture_path,
                            )
                            .await
                            {
                                Ok(_) => {}
                                Err(e) => {
                                    error!("Failed to download texture: {:?}", e)
                                }
                            }

                            // Download the mesh itself
                            let render_objects = match download_scene_group(
                                &scene_group,
                                &server_endpoint,
                                &texture_path,
                            )
                            .await
                            {
                                Ok(objects) => objects,
                                Err(e) => {
                                    error!("{:?}", e);
                                    return;
                                }
                            };

                            let json_path = format!(
                                "{:?}_{}",
                                scene_group.parts[0].sculpt.texture,
                                scene_group.parts[0].metadata.name
                            );

                            // write the json
                            let json = match write_json(&render_objects, msg.agent_id, json_path) {
                                Ok(json) => json,
                                Err(e) => {
                                    error!("Failed to write json: {:?}", e);
                                    return;
                                }
                            };

                            // add the object to the avatar
                            addr.do_send(AddObjectToAvatar {
                                object: OutfitObject::MeshObject(json),
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

impl Handler<HandleNewAvatar> for Mailbox {
    type Result = ();
    fn handle(&mut self, msg: HandleNewAvatar, ctx: &mut Self::Context) -> Self::Result {
        if let Some(session) = self.session.as_mut() {
            let addr = ctx.address();
            if session.agent_id == msg.avatar.agent_id {
                addr.do_send(SendUIMessage {
                    ui_message: UIMessage::new_camera_position(CameraPosition {
                        position: msg.avatar.position,
                    }),
                });

                if session.inventory_data.inventory_init {
                    session.avatars.insert(msg.avatar.agent_id, msg.avatar);
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
            } else {
                // TODO: handle non-user avatar updates
                warn!("Non-user avatar updates not yet supported...");
            }
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

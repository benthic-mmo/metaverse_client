use super::session::Mailbox;
use crate::session::SendUIMessage;
use actix::{AsyncContext, Handler, Message, WrapFuture};
use benthic_protocol::messages::ui::camera_position::CameraPosition;
use benthic_protocol::messages::ui::mesh_update::{MeshType, MeshUpdate};
use benthic_protocol::messages::ui::play_animation::PlayAnimation;
use benthic_protocol::messages::ui::ui_messages::UIMessage;
use benthic_protocol::session::create_agent_animation_dir;
use benthic_protocol::skeleton::JointName;
use glam::{Quat, Vec3};
use log::{error, warn};
use metaverse_avatar::animation::build_animation;
use metaverse_avatar::avatar::Avatar;
use metaverse_avatar::avatar::OutfitObject;
use metaverse_avatar::avatar_object_handler::AvatarState::FullyLoaded;
use metaverse_avatar::avatar_object_handler::{
    AvatarType, add_object_to_avatar, finalize_avatar, init_avatar,
};
use metaverse_avatar::errors::AvatarError::{self};
use metaverse_cache::agent::sqlite_update_avatar;
use metaverse_cache::avatar::cache_current_outfit;
use metaverse_messages::http::capabilities::Capability;
use metaverse_messages::udp::agent::avatar_animation::AvatarAnimation;
use metaverse_messages::udp::agent::avatar_appearance::AvatarAppearance;
use metaverse_messages::utils::object_types::ObjectType;
use metaverse_objects::avatar_asset::download_asset_objects;
use std::collections::BTreeSet;
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
impl Handler<DownloadAgentAsset> for Mailbox {
    type Result = ();
    fn handle(&mut self, msg: DownloadAgentAsset, ctx: &mut Self::Context) -> Self::Result {
        let Some(session) = self.session.as_mut() else {
            return;
        };
        let db_conn = session.inventory_db_connection.clone();
        let server_endpoint = session
            .capability_urls
            .get(&Capability::ViewerAsset)
            .unwrap()
            .to_string();
        let object_type = msg.item_type;
        let asset_id = msg.asset_id;
        let agent_id = msg.agent_id;

        let addr = ctx.address();
        ctx.spawn(
            async move {
                match download_asset_objects(
                    &server_endpoint,
                    db_conn,
                    object_type,
                    asset_id,
                    agent_id,
                )
                .await
                {
                    Ok(json_path) => {
                        addr.do_send(AddObjectToAvatar {
                            object: OutfitObject::MeshObject(json_path),
                            agent_id,
                        });
                    }
                    Err(e) => {
                        error!("{:?}", e)
                    }
                };
            }
            .into_actor(self),
        );
    }
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
impl Handler<AddObjectToAvatar> for Mailbox {
    type Result = ();
    fn handle(&mut self, msg: AddObjectToAvatar, ctx: &mut Self::Context) -> Self::Result {
        let Some(session) = self.session.as_mut() else {
            return;
        };
        match add_object_to_avatar(session, msg.agent_id, msg.object) {
            Ok(FullyLoaded) => {
                ctx.address().do_send(FinalizeAvatar {
                    agent_id: msg.agent_id,
                });
            }
            Ok(_) => {}
            Err(e) => {
                error!("{:?}", e);
            }
        };
    }
}

/// Message to spawn a new avatar
///
/// Avatars are added to the scene via ObjectUpdate packet with minimal information aside from the
/// ID. This adds those IDs to the session and begins downloading their appearances, or loads from
/// the cache.
///
/// # Cause
/// - [`HandleObjectUpdate`]
///
/// # Effect
/// - Resends a [`HandleNewAvatar`] message if the inventory is not yet loaded
/// - If the avatar is the current player's avatar
///    - Dispatches a [`CameraPosition`] message
///    - If the avatar is in the cache:
///     - Dispatches a [`LoadFromCache`] message to skip asset downloading.
///    - else:
///     - Dispatches a [`DownloadAgentAsset`] message for each object in the outfit
///     - Dispatches a [`AddObjectToAvatar`] message for each bodypart in the outfit
#[derive(Debug, Message)]
#[rtype(result = "()")]
pub struct HandleNewAvatar(pub Avatar);
impl Handler<HandleNewAvatar> for Mailbox {
    type Result = ();
    fn handle(&mut self, msg: HandleNewAvatar, ctx: &mut Self::Context) -> Self::Result {
        let Some(session) = self.session.as_mut() else {
            return;
        };
        let addr = ctx.address();
        match init_avatar(session, &msg.0) {
            Ok(user_type) => {
                match user_type {
                    AvatarType::User => {
                        addr.do_send(SendUIMessage {
                            ui_message: UIMessage::new_camera_position(CameraPosition {
                                position: msg.0.position,
                            }),
                        });
                        let agent_id = session.agent_id;
                        let db_conn = session.inventory_db_connection.clone();
                        ctx.spawn(
                            async move {
                                match cache_current_outfit(db_conn, agent_id).await {
                                    Ok((avatar, outfit_items)) => {
                                        if addr
                                            .send(SetOutfitSize {
                                                agent_id,
                                                outfit_size: outfit_items.len(),
                                            })
                                            .await
                                            .is_err()
                                        {
                                            error!("Failed to set outfit size for {:?}", agent_id);
                                            return;
                                        }

                                        if let Some(avatar) = avatar {
                                            addr.do_send(LoadFromCache { avatar });
                                            return;
                                        }
                                        for item in outfit_items {
                                            match item.item_type {
                                                ObjectType::Object => {
                                                    addr.do_send(DownloadAgentAsset {
                                                        asset_id: item.asset_id,
                                                        item_type: item.item_type,
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
                                        error!("{:?}", e)
                                    }
                                }
                            }
                            .into_actor(self),
                        );
                    }
                    AvatarType::NonUser => {
                        //TODO: handle nonuser updates
                    }
                }
            }
            Err(e) => match e {
                AvatarError::InventoryUninitialized { .. } => {
                    warn!("{:?}, Requeueing avatar download...", e);
                    ctx.notify_later(msg, Duration::from_secs(1));
                }
                e => {
                    error!("{:?}", e)
                }
            },
        };
    }
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
/// - Dispatches a [`RenderAvatar`] message to render the finalized avatar
/// - Dispatches a [`SetAvatarGlbPath`] message to ensure the GLB path gets set
#[derive(Debug, Message)]
#[rtype(result = "()")]
pub struct FinalizeAvatar {
    /// the avatar object to generate
    pub agent_id: Uuid,
}
impl Handler<FinalizeAvatar> for Mailbox {
    type Result = ();
    fn handle(&mut self, msg: FinalizeAvatar, ctx: &mut Self::Context) -> Self::Result {
        let Some(session) = self.session.as_mut() else {
            return;
        };

        let Some(avatar) = session.avatars.get_mut(&msg.agent_id) else {
            error!("Avatar {:?} not found in avatar cache", msg.agent_id);
            return;
        };

        let agent_id = msg.agent_id;
        let skeleton = avatar.skeleton.clone();
        let used_joints = avatar.used_joints.clone();
        let items = avatar.items.clone();
        let db_conn = session.inventory_db_connection.clone();
        let position = avatar.position;
        let mut avatar = avatar.clone();

        let addr = ctx.address();
        ctx.spawn(
            async move {
                match finalize_avatar(agent_id, skeleton, used_joints.clone(), items).await {
                    Ok(glb_path) => {
                        addr.do_send(SetAvatarGlbPath {
                            path: glb_path.clone(),
                            agent_id,
                        });
                        // edit the temp avatar for the sqlite update.
                        avatar.path = Some(glb_path.clone());
                        if let Err(e) = sqlite_update_avatar(&db_conn, avatar).await {
                            error!("{:?}", e);
                        }
                        addr.do_send(RenderAvatar {
                            message: MeshUpdate {
                                position,
                                scale: Vec3::ONE,
                                rotation: Quat::IDENTITY,
                                parent: None,
                                scene_id: None,
                                path: glb_path,
                                mesh_type: MeshType::Avatar,
                                id: Some(msg.agent_id),
                            },
                            agent_id,
                            skeleton: used_joints,
                        });
                    }
                    Err(e) => {
                        error!("{:?}", e)
                    }
                };
            }
            .into_actor(self),
        );
    }
}

#[derive(Debug, Message)]
#[rtype(result = "()")]
struct LoadFromCache {
    avatar: Avatar,
}
impl Handler<LoadFromCache> for Mailbox {
    type Result = ();
    fn handle(&mut self, msg: LoadFromCache, ctx: &mut Self::Context) -> Self::Result {
        let Some(session) = self.session.as_mut() else {
            return;
        };
        // insert the avatar to the session
        session
            .avatars
            .insert(msg.avatar.agent_id, msg.avatar.clone());

        // render the cached avatar
        ctx.address().do_send(RenderAvatar {
            message: MeshUpdate {
                position: msg.avatar.position,
                scale: Vec3::ONE,
                rotation: Quat::IDENTITY,
                parent: None,
                scene_id: None,
                path: msg.avatar.path.unwrap(),
                mesh_type: MeshType::Avatar,
                id: Some(msg.avatar.agent_id),
            },
            agent_id: msg.avatar.agent_id,
            skeleton: msg.avatar.used_joints,
        });
    }
}

/// Message to handle an updated animation
///
/// # Cause
/// - Avatar Appearance packet received from UDP socket
///
/// # Effect
/// - Dispatches a [`PlayAnimation`] message to inform the UI of a new animation
#[derive(Debug, Message)]
#[rtype(result = "()")]
pub struct HandleNewAvatarAnimation {
    /// the avatar appearance data to handle
    pub avatar_animation: AvatarAnimation,
}
impl Handler<HandleNewAvatarAnimation> for Mailbox {
    type Result = ();
    fn handle(&mut self, msg: HandleNewAvatarAnimation, ctx: &mut Self::Context) -> Self::Result {
        let Some(session) = self.session.as_mut() else {
            return;
        };

        let avatar = match session.avatars.get(&msg.avatar_animation.sender_id) {
            Some(avatar) => {
                if !avatar.fully_loaded {
                    warn!(
                        "Animation targeting player {:?} is not yet fully loaded. Queueing animation...",
                        msg.avatar_animation.sender_id
                    );
                    ctx.notify_later(msg, Duration::from_secs(1));
                    return;
                }
                avatar
            }
            None => {
                warn!(
                    "Animation targeting player {:?}, not yet in scene. Queueing animation...",
                    msg.avatar_animation.sender_id
                );
                ctx.notify_later(msg, Duration::from_secs(1));
                return;
            }
        };

        let animations = msg.avatar_animation.animations;
        let used_joints = avatar.used_joints.clone();
        let target_skeleton = avatar.skeleton.clone();
        let Ok(out_dir) = create_agent_animation_dir(&avatar.agent_id) else {
            error!("Failed to create agent animation directory");
            return;
        };

        let sender_id = msg.avatar_animation.sender_id;
        let addr = ctx.address();
        ctx.spawn(
            async move {
                match build_animation(animations, used_joints, target_skeleton, out_dir).await {
                    Ok(mesh_paths) => {
                        for path in mesh_paths {
                            addr.do_send(SendUIMessage {
                                ui_message: UIMessage::new_play_animation(PlayAnimation {
                                    player_id: sender_id,
                                    animation_path: path,
                                }),
                            });
                        }
                    }
                    Err(e) => {
                        error!("{:?}", e)
                    }
                };
            }
            .into_actor(self),
        );
    }
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

/// Helper message to ensure all avatars are marked as fully loaded once the mesh render is triggered.
///
/// # Cause
/// - [`FinalizeAvatar`]
/// - [`LoadFromCache`]
///
/// # Effects
/// - Dispatches a [`MeshUpdate`] message to inform the UI of an update
#[derive(Debug, Message)]
#[rtype(result = "()")]
pub struct RenderAvatar {
    message: MeshUpdate,
    agent_id: Uuid,
    skeleton: BTreeSet<JointName>,
}
impl Handler<RenderAvatar> for Mailbox {
    type Result = ();
    fn handle(&mut self, msg: RenderAvatar, ctx: &mut Self::Context) -> Self::Result {
        if let Some(session) = self.session.as_mut()
            && let Some(avatar) = session.avatars.get_mut(&msg.agent_id)
        {
            avatar.used_joints = msg.skeleton;
            let addr = ctx.address();

            addr.do_send(SendUIMessage {
                ui_message: UIMessage::new_mesh_update(msg.message),
            });

            avatar.fully_loaded = true;
        }
    }
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
impl Handler<SetOutfitSize> for Mailbox {
    type Result = ();
    fn handle(&mut self, msg: SetOutfitSize, _ctx: &mut Self::Context) -> Self::Result {
        if let Some(session) = self.session.as_mut()
            && let Some(avatar) = session.avatars.get_mut(&msg.agent_id)
        {
            avatar.outfit_size = msg.outfit_size;
        }
    }
}

/// Simple helper function to ensure that the avatar's GLB path is set correctly
///
/// #Cause
/// - [`FinalizeAvatar`]
#[derive(Debug, Message)]
#[rtype(result = "()")]
pub struct SetAvatarGlbPath {
    /// GLB path
    pub path: PathBuf,
    /// Agent ID to set
    pub agent_id: Uuid,
}
impl Handler<SetAvatarGlbPath> for Mailbox {
    type Result = ();
    fn handle(&mut self, msg: SetAvatarGlbPath, _ctx: &mut Self::Context) -> Self::Result {
        if let Some(session) = self.session.as_mut()
            && let Some(avatar) = session.avatars.get_mut(&msg.agent_id)
        {
            avatar.path = Some(msg.path)
        }
    }
}

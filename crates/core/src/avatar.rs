use super::session::Mailbox;
use crate::COORDINATE_CONVERSION;
use crate::session::SendUIMessage;
use crate::transport::udp_handler::classify_object_update;
use actix::{AsyncContext, Handler, Message, WrapFuture};
use benthic_protocol::avatar::skeleton::Skeleton;
use benthic_protocol::avatar::skeleton::SkinJoint;
use benthic_protocol::avatar::types::AttachmentData;
use benthic_protocol::messages::agent::avatar_update::AvatarUpdate;
use benthic_protocol::messages::agent::play_animation::PlayAnimation;
use benthic_protocol::messages::camera::camera_position::CameraPosition;
use benthic_protocol::messages::ui_protocol::UIMessage;
use benthic_protocol::session::storage::cache_enabled;
use benthic_protocol::session::storage::create_sub_agent_dir;
use glam::{Quat, Vec3};
use log::{error, info, warn};
use metaverse_avatar::animation::build_animation;
use metaverse_avatar::avatar::Avatar;
use metaverse_avatar::avatar_object_handler::finalize_avatar;
use metaverse_avatar::skeleton::update_global_avatar_skeleton;
use metaverse_messages::http::capabilities::Capability;
use metaverse_messages::http::scene::SceneGroup;
use metaverse_messages::udp::agent::avatar_animation::AvatarAnimation;
use metaverse_messages::udp::agent::avatar_appearance::AvatarAppearance;
use metaverse_messages::utils::object_types::ObjectType;
use metaverse_objects::download_asset::download_object;
use metaverse_objects::object_updates::AvatarAttachment;
use metaverse_objects::object_updates::AvatarObjectData;
use std::collections::BTreeSet;
use std::path::PathBuf;
use std::time::SystemTime;
use uuid::Uuid;

#[derive(Message, Clone, Debug)]
#[rtype(result = "()")]
pub struct AvatarObjectUpdate {
    /// ID of the avatar
    pub id: Uuid,

    pub local_id: u32,
    /// Position in space of the avatar
    pub position: Vec3,
}
impl Handler<AvatarObjectUpdate> for Mailbox {
    type Result = ();

    fn handle(&mut self, msg: AvatarObjectUpdate, ctx: &mut Self::Context) {
        let Some(session) = self.session.as_mut() else {
            return;
        };

        // If this is the user's avatar, use the special handler.
        if session.agent_id == msg.id {
            ctx.address().do_send(UserAvatarObjectUpdate(msg));
            return;
        }

        let avatar = Avatar::new(msg.id, msg.position);
        session.avatars.insert(msg.id, avatar);
        self.pending_avatars.insert(msg.id);
        println!(
            "!!!!!!!!!!!!!!!!!!!!!!! PENDING AVATARS {:?}:",
            self.pending_avatars
        );

        let cache = self.cache.clone();
        ctx.spawn(
            async move {
                if let Err(e) = cache.object.update_avatar(msg.local_id, msg.id).await {
                    error!("Failed to cache avatar update: {e}");
                }
            }
            .into_actor(self),
        );
    }
}

#[derive(Message, Clone, Debug)]
#[rtype(result = "()")]
pub struct UserAvatarObjectUpdate(AvatarObjectUpdate);
impl Handler<UserAvatarObjectUpdate> for Mailbox {
    type Result = ();
    fn handle(&mut self, msg: UserAvatarObjectUpdate, ctx: &mut Self::Context) {
        let Some(session) = self.session.as_mut() else {
            return;
        };
        let Some(endpoint) = session
            .capability_urls
            .get(&Capability::ViewerAsset)
            .cloned()
        else {
            info!("ViewerAsset capability unavailable. Queuing avatar update.");
            self.pending_avatar_objects.push(msg.0);
            return;
        };

        if !session.inventory_data.current_outfit_init {
            info!("Current outfit not initialized. Adding avatar update to queue");
            self.pending_avatar_objects.push(msg.0);
            return;
        }

        let avatar = Avatar::new(msg.0.id, msg.0.position);
        session.avatars.insert(msg.0.id, avatar);

        let addr = ctx.address();
        addr.do_send(SendUIMessage {
            ui_message: UIMessage::new_camera_position(CameraPosition {
                position: COORDINATE_CONVERSION.mul_vec3(msg.0.position),
            }),
        });

        let agent_id = session.agent_id;
        let current_outfit_id = session.inventory_data.current_outfit_root;
        let cache = self.cache.clone();
        let inventory = session.inventory.clone();
        let out_dir = session.share_dir_root.clone();
        let client = self.http_client.clone();
        let rate_limiter = self.rate_limiter.clone();

        ctx.spawn(
            async move {
                let (cached_avatar, outfit_items) = if cache_enabled() {
                    match cache
                        .avatar
                        .current_outfit(agent_id, inventory.clone(), current_outfit_id)
                        .await
                    {
                        Ok(result) => result,
                        Err(e) => {
                            error!("Failed to retrieve current outfit from cache: {e:?}");
                            return;
                        }
                    }
                } else {
                    match inventory.get_current_outfit(current_outfit_id).await {
                        Ok(items) => (None, items),
                        Err(e) => {
                            error!("Failed to retrieve current outfit: {e:?}");
                            return;
                        }
                    }
                };

                if let Some(avatar) = cached_avatar {
                    addr.do_send(LoadFromCache { avatar });
                    return;
                }

                let asset_dir = match create_sub_agent_dir(&out_dir, &agent_id.to_string()) {
                    Ok(path) => path,
                    Err(e) => {
                        error!("Failed to create avatar asset directory: {e}");
                        return;
                    }
                };

                let mut scenes = Vec::new();
                for item in outfit_items {
                    match item.item_type {
                        ObjectType::Object => {
                            let object_path = asset_dir.join(format!("{}.xml", item.asset_id));

                            if let Err(e) = download_object(
                                item.asset_id,
                                &endpoint,
                                &object_path,
                                client.clone(),
                                rate_limiter.clone(),
                            )
                            .await
                            {
                                error!(
                                    "Failed to download inventory object {}: {e}",
                                    item.asset_id
                                );
                                continue;
                            }

                            let bytes = match tokio::fs::read(&object_path).await {
                                Ok(bytes) => bytes,
                                Err(e) => {
                                    error!(
                                        "Failed to read inventory object {}: {e}",
                                        item.asset_id
                                    );
                                    continue;
                                }
                            };

                            match SceneGroup::from_xml(&bytes) {
                                Ok(group) => scenes.extend(group.parts),
                                Err(e) => {
                                    error!(
                                        "Failed to parse inventory object {}: {e}",
                                        item.asset_id
                                    );
                                }
                            }
                        }
                        _ => {
                            addr.do_send(AddNonMeshToAvatar {
                                agent_id,
                                object_type: item.item_type,
                            });
                        }
                    }
                }

                let outfit_size = scenes.len();
                if addr
                    .send(SetOutfitSize {
                        agent_id,
                        outfit_size,
                    })
                    .await
                    .is_err()
                {
                    error!("Failed to set outfit size for {agent_id:?}");
                    return;
                }

                for scene in scenes {
                    let attachment = AvatarAttachment {
                        agent_id,
                        attachment: AttachmentData {
                            attach_point: scene.shape.last_attach_point,
                            position: scene.attached_pos,
                            rotation: scene.rotation_offset,
                            scale: scene.scale,
                        },
                    };

                    if let Err(e) =
                        classify_object_update(scene, addr.clone(), Some(attachment)).await
                    {
                        error!("Failed to classify avatar attachment: {e}");
                    }
                }
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
/// - [`DownloadAgentAsset`]
/// - [`AvatarObjectUpdate`]
///
/// # Effects
/// - Dispatches a [`FinalizeAvatar`] message if the avatar's outfit has all items
#[derive(Debug, Message)]
#[rtype(result = "()")]
pub struct AddObjectToAvatar(pub AvatarObjectData);
impl Handler<AddObjectToAvatar> for Mailbox {
    type Result = ();
    fn handle(&mut self, msg: AddObjectToAvatar, ctx: &mut Self::Context) {
        let Some(session) = self.session.as_mut() else {
            return;
        };
        let Some(avatar) = session.avatars.get_mut(&msg.0.agent_id) else {
            error!("Avatar not present in session. Aborting.");
            return;
        };

        // Update the global skeleton before adding to the items list.
        if let Some(skeleton) = msg.0.skeleton_update {
            update_global_avatar_skeleton(avatar, &skeleton);
        }

        avatar.items.push(msg.0.json_path);
        avatar.last_update = SystemTime::now();

        if avatar.items.len() == avatar.outfit_size {
            ctx.address().do_send(FinalizeAvatar {
                agent_id: msg.0.agent_id,
            });
        }
    }
}

#[derive(Debug, Message)]
#[rtype(result = "()")]
pub struct AddNonMeshToAvatar {
    object_type: ObjectType,
    agent_id: Uuid,
}
impl Handler<AddNonMeshToAvatar> for Mailbox {
    type Result = ();
    fn handle(&mut self, msg: AddNonMeshToAvatar, ctx: &mut Self::Context) {
        //TODO: hande this.
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
        let position = COORDINATE_CONVERSION.mul_vec3(avatar.position);
        let cache = self.cache.clone();
        let base_dir = session.share_dir_root.clone();
        let mut avatar = avatar.clone();

        let addr = ctx.address();
        ctx.spawn(
            async move {
                match finalize_avatar(agent_id, skeleton, used_joints.clone(), items, base_dir)
                    .await
                {
                    Ok((glb_path, skeleton)) => {
                        addr.do_send(SetAvatarGlbPath {
                            path: glb_path.clone(),
                            agent_id,
                        });
                        addr.do_send(SetAvatarFinalSkeleton {
                            final_skeleton: skeleton,
                            agent_id,
                        });
                        // edit the temp avatar for the sqlite update.
                        avatar.path = Some(glb_path.clone());
                        if cache_enabled()
                            && let Err(e) = cache.avatar.update(avatar).await
                        {
                            error!("FinalizeAvatar error: {:?}", e);
                        }
                        addr.do_send(RenderAvatar {
                            message: AvatarUpdate {
                                position,
                                rotation: Quat::IDENTITY,
                                path: glb_path,
                                id: Some(msg.agent_id),
                            },
                            agent_id,
                            skeleton: used_joints,
                        });
                    }
                    Err(e) => {
                        error!("Finalize avatar error: {:?}", e)
                    }
                };
            }
            .into_actor(self),
        );
    }
}

/// Handles adding pre-cached avatars to the session
///
/// Ensures that the session's local state contains all of the required objects
/// TODO: does not currently do that. This needs to be fixed ASAP.
///
/// # Cause
/// - [`AvatarObjectUpdate`]
///
/// # Effect
/// - [`RenderAvatar`]
#[derive(Debug, Message)]
#[rtype(result = "()")]
pub struct LoadFromCache {
    /// The avatar to load from cache.
    pub avatar: Avatar,
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

        let position = COORDINATE_CONVERSION.mul_vec3(msg.avatar.position);
        // render the cached avatar
        ctx.address().do_send(RenderAvatar {
            message: AvatarUpdate {
                position,
                rotation: Quat::IDENTITY,
                path: msg.avatar.path.unwrap(),
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
/// - Avatar Animation packet received from UDP socket
///
/// # Effect
/// - Dispatches a [`PlayAnimation`] message to inform the UI of a new animation
#[derive(Debug, Message)]
#[rtype(result = "()")]
pub struct HandleNewAvatarAnimation(pub AvatarAnimation);
impl Handler<HandleNewAvatarAnimation> for Mailbox {
    type Result = ();
    fn handle(&mut self, msg: HandleNewAvatarAnimation, ctx: &mut Self::Context) -> Self::Result {
        let Some(session) = self.session.as_mut() else {
            return;
        };
        if msg.0.sender_id != session.agent_id {
            //TODO: IMPLEMENT NON-USER ANIMATIONS
            warn!("Non-user animation triggered. Currently unimplemented. Dropping animaiton");
            return;
        }

        let avatar = match session.avatars.get(&msg.0.sender_id) {
            Some(avatar) => {
                if !avatar.fully_loaded {
                    info!("Avatar not fully loaded. Adding to queue.");
                    self.pending_animations
                        .entry(msg.0.sender_id)
                        .or_default()
                        .push(msg);

                    return;
                }
                avatar
            }
            None => {
                info!("Avatar not yet in scene. Adding to queue.");
                self.pending_animations
                    .entry(msg.0.sender_id)
                    .or_default()
                    .push(msg);

                return;
            }
        };

        let animations = msg.0.animations;
        let used_joints = avatar.used_joints.clone();
        let target_skeleton = avatar.skeleton.clone();
        let out_dir = session.share_dir_root.clone();
        let agent_id = session.agent_id;

        let sender_id = msg.0.sender_id;
        let addr = ctx.address();
        ctx.spawn(
            async move {
                match build_animation(animations, used_joints, target_skeleton, out_dir, agent_id)
                    .await
                {
                    Ok(mesh_paths) => {
                        for path in mesh_paths {
                            println!("PLAYING ANIMATION AT PATH: {:?}", path);
                            addr.do_send(SendUIMessage {
                                ui_message: UIMessage::new_play_animation(PlayAnimation {
                                    player_id: sender_id,
                                    animation_path: path,
                                }),
                            });
                        }
                    }
                    Err(e) => {
                        error!("Build animation failed: {:?}", e)
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
        // TODO: implement this. AvatarAppearance packets change skeleton deform joint positions to change
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
/// - Dispatches a [`AvatarUpdate`] message to inform the UI of an update
#[derive(Debug, Message)]
#[rtype(result = "()")]
pub struct RenderAvatar {
    message: AvatarUpdate,
    agent_id: Uuid,
    skeleton: BTreeSet<SkinJoint>,
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
                ui_message: UIMessage::new_avatar_update(msg.message),
            });

            avatar.fully_loaded = true;
            if let Some(animations) = self.pending_animations.remove(&msg.agent_id) {
                for animation in animations {
                    ctx.address().do_send(animation);
                }
            }
        }
    }
}

/// Simple helper message to set the outfit size of the avatar
///
/// This must be set in order to trigger a render. Without this, the avatar doesn't know when the
/// outfit is finished loading in.
///
/// # Cause
/// - [`AvatarObjectUpdate`]
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

/// Simple helper message to ensure that the avatar's GLB path is set correctly
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
/// Simple helper message to ensure that the avatar's skeleton is updated after finalizing
///
/// #Cause
/// - [`FinalizeAvatar`]
#[derive(Debug, Message)]
#[rtype(result = "()")]
pub struct SetAvatarFinalSkeleton {
    /// Skeleton
    pub final_skeleton: Skeleton,
    /// Agent ID to set
    pub agent_id: Uuid,
}
impl Handler<SetAvatarFinalSkeleton> for Mailbox {
    type Result = ();
    fn handle(&mut self, msg: SetAvatarFinalSkeleton, _ctx: &mut Self::Context) -> Self::Result {
        if let Some(session) = self.session.as_mut()
            && let Some(avatar) = session.avatars.get_mut(&msg.agent_id)
        {
            avatar.skeleton = msg.final_skeleton
        }
    }
}

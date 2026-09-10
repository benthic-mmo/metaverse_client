use super::session::Mailbox;
use crate::errors::FilterAnimationError;
use crate::initialize::{
    create_agent_animation_dir, create_filtered_animation_dir, create_sub_agent_dir,
};
use crate::session::SendUIMessage;
use crate::transport::http_handler::{
    download_asset, download_object, download_scene_group, download_texture,
};
use actix::{AsyncContext, Handler, Message, WrapFuture};
use benthic_asset_pipeline::generated::DEFAULT_SKELETON;
use benthic_asset_pipeline::generated_asset_path;
use benthic_protocol::default_animations::{AnimationClip, BindJoint, DefaultAnimation};
use benthic_protocol::messages::ui::camera_position::CameraPosition;
use benthic_protocol::messages::ui::mesh_update::{MeshType, MeshUpdate};
use benthic_protocol::messages::ui::play_animation::PlayAnimation;
use benthic_protocol::messages::ui::ui_messages::UIMessage;
use benthic_protocol::render_data::{AvatarObject, RenderObject};
use benthic_protocol::skeleton::{JointName, Skeleton};
use glam::{Quat, Vec3};
use indexmap::IndexMap;
use log::{error, warn};
use metaverse_agent::avatar::Avatar;
use metaverse_agent::avatar::OutfitObject;
use metaverse_agent::skeleton::update_global_avatar_skeleton;
use metaverse_cache::agent::sqlite_get_current_outfit;
use metaverse_cache::agent::{sqlite_get_avatar, sqlite_get_current_avatar_version};
use metaverse_cache::agent::{sqlite_get_current_outfit_version, sqlite_insert_avatar};
use metaverse_cache::agent::{sqlite_update_avatar, sqlite_update_outfit_item_json_path};
use metaverse_mesh::animation::generate::generate_gltf_animation;
use metaverse_mesh::mesh::generate::generate_skinned_mesh;
use metaverse_messages::http::capabilities::Capability;
use metaverse_messages::udp::agent::avatar_animation::AvatarAnimation;
use metaverse_messages::udp::agent::avatar_appearance::AvatarAppearance;
use metaverse_messages::utils::object_types::ObjectType;
use serde::Serialize;
use std::collections::{BTreeSet, HashSet};
use std::fs::{self, File};
use std::hash::{DefaultHasher, Hash, Hasher};
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
pub struct HandleNewAvatar {
    /// the avatar object to generate
    pub avatar: Avatar,
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
#[derive(Debug, Message)]
#[rtype(result = "()")]
pub struct FinalizeAvatar {
    /// the avatar object to generate
    pub agent_id: Uuid,
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

#[derive(Debug, Message)]
#[rtype(result = "()")]
struct LoadFromCache {
    avatar: Avatar,
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

impl Handler<HandleNewAvatar> for Mailbox {
    type Result = ();
    fn handle(&mut self, msg: HandleNewAvatar, ctx: &mut Self::Context) -> Self::Result {
        if let Some(session) = self.session.as_mut() {
            let addr = ctx.address();
            if session.agent_id == msg.avatar.agent_id {
                let pos = msg.avatar.position;
                let position = Vec3 {
                    x: pos.x,
                    y: pos.y,
                    z: pos.z,
                };
                addr.do_send(SendUIMessage {
                    ui_message: UIMessage::new_camera_position(CameraPosition { position }),
                });

                if session.inventory_data.inventory_init {
                    session.avatars.insert(msg.avatar.agent_id, msg.avatar);
                    let agent_id = session.agent_id;
                    let db_conn = self.inventory_db_connection.clone();
                    ctx.spawn(
                        async move {
                            match sqlite_get_current_outfit(&db_conn).await {
                                Ok(outfit) => {
                                    let current_outfit_version =
                                        match sqlite_get_current_outfit_version(&db_conn).await {
                                            Ok(version) => version,
                                            Err(e) => {
                                                error!(
                                                    "Failed to get current outfit version {:?}",
                                                    e
                                                );
                                                0
                                            }
                                        };
                                    let avatar_current_version =
                                        match sqlite_get_current_avatar_version(
                                            &db_conn,
                                            agent_id.to_string(),
                                        )
                                        .await
                                        {
                                            Ok(version) => version,
                                            Err(e) => {
                                                error!(
                                                    "Failed to get current avatar version {:?}",
                                                    e
                                                );
                                                0
                                            }
                                        };

                                    match sqlite_insert_avatar(&db_conn, agent_id, current_outfit_version).await{
                                        Ok(_) =>{},
                                        Err(e) => {error!("Failed to insert new avatar to cache: {:?}", e)}
                                    }

                                    if current_outfit_version == avatar_current_version {
                                        match sqlite_get_avatar(&db_conn, agent_id).await {
                                            Ok(avatar) => {
                                                addr.do_send(LoadFromCache{avatar});
                                                return;
                                            }
                                            Err(e) => {
                                                error!("Failed to retrieve avatar data from cache: {:?}", e);
                                            }
                                        }
                                    }
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

impl Handler<LoadFromCache> for Mailbox {
    type Result = ();
    fn handle(&mut self, msg: LoadFromCache, ctx: &mut Self::Context) -> Self::Result {
        if let Some(session) = self.session.as_mut() {
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
        };
    }
}

impl Handler<SetOutfitSize> for Mailbox {
    type Result = ();
    fn handle(&mut self, msg: SetOutfitSize, _ctx: &mut Self::Context) -> Self::Result {
        if let Some(session) = self.session.as_mut()
            && let Some(agent) = session.avatars.get_mut(&msg.agent_id)
        {
            agent.outfit_size = msg.outfit_size;
        }
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
            let db_conn = self.inventory_db_connection.clone();
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
                            let texture_path = if texture_path.exists() {
                                texture_path
                            } else {
                                match download_texture(
                                    ObjectType::Texture.to_string(),
                                    texture_id,
                                    &server_endpoint,
                                    &texture_path,
                                )
                                .await
                                {
                                    Ok(_) => texture_path,
                                    Err(e) => {
                                        error!("Failed to download texture: {:?}", e);

                                        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                                            .join("assets")
                                            .join("textures")
                                            .join("benthic_default_texture.png")
                                    }
                                }
                            };

                            // Download the SceneGroup which also downloads the full mesh
                            // information.
                            // this returns a vec of RenderObjects which are stored as JSON to save memory.
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

                            let json = if std::path::Path::new(&json_path).exists() {
                                PathBuf::from(json_path.clone())
                            } else {
                                match write_json(&render_objects, msg.agent_id, &json_path) {
                                    Ok(json) => json,
                                    Err(e) => {
                                        error!("Failed to write json: {:?}", e);
                                        return;
                                    }
                                }
                            };

                            match sqlite_update_outfit_item_json_path(
                                &db_conn,
                                msg.asset_id,
                                &json_path.clone(),
                            )
                            .await
                            {
                                Ok(_) => {}
                                Err(e) => {
                                    warn!("Failed to update sqlite: {:?}", e)
                                }
                            };

                            // add the object to the avatar
                            addr.do_send(AddObjectToAvatar {
                                object: OutfitObject::MeshObject(json),
                                agent_id: msg.agent_id,
                            });
                        }
                        Err(e) => {
                            error!("Object contained no SceneGroup: {:?}, {:?}", e, msg);
                        }
                    }
                }
                .into_actor(self),
            );
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
                        let file = fs::File::open(path)
                            .unwrap_or_else(|e| panic!("Failed to open {:?}: {}", path, e));
                        let parts: Vec<RenderObject> = serde_json::from_reader(file)
                            .unwrap_or_else(|e| panic!("Failed to read serde {:?}: {}", path, e));

                        if let Some(skin) = &parts[0].skin {
                            update_global_avatar_skeleton(avatar, &skin.skeleton);
                        }
                    }
                    _ => {
                        //TODO: unimplemented!
                        //warn!("Avatars wearing non-mesh objects are currently not supported.")
                    }
                }

                avatar.items.push(msg.object);

                if avatar.items.len() == avatar.outfit_size {
                    ctx.address().do_send(FinalizeAvatar {
                        agent_id: avatar.agent_id,
                    });
                }
            } else {
                warn!("Agent not found for agent_id {:?}", msg.agent_id);
            }
        }
    }
}

impl Handler<FinalizeAvatar> for Mailbox {
    type Result = ();
    fn handle(&mut self, msg: FinalizeAvatar, ctx: &mut Self::Context) -> Self::Result {
        if let Some(session) = self.session.as_mut()
            && let Some(avatar) = session.avatars.get_mut(&msg.agent_id)
        {
            let addr = ctx.address();

            let agent_id = avatar.agent_id;
            let position = avatar.position;
            let skeleton = avatar.skeleton.clone();
            let used_joints = avatar.used_joints.clone();
            let items = avatar.items.clone();

            let mut avatar_clone = avatar.clone();
            let db_conn = self.inventory_db_connection.clone();

            avatar.fully_loaded = true;
            ctx.spawn(
                async move {
                    let json_paths: Vec<PathBuf> = items
                        .into_iter()
                        .filter_map(|item| {
                            if let OutfitObject::MeshObject(path) = item {
                                Some(path)
                            } else {
                                None
                            }
                        })
                        .collect();

                    let avatar_object = AvatarObject {
                        objects: json_paths,
                        global_skeleton: skeleton,
                        used_joints: used_joints.clone(),
                    };

                    let json_path_str = agent_id.to_string();
                    let json_path = PathBuf::from(&json_path_str);

                    let json_path = if json_path.exists() {
                        json_path.clone()
                    } else {
                        match write_json(&avatar_object, agent_id, &json_path_str) {
                            Ok(p) => p,
                            Err(e) => {
                                error!("write_json failed: {:?}", e);
                                return;
                            }
                        }
                    };

                    let base_dir = match create_sub_agent_dir(&msg.agent_id.to_string()) {
                        Ok(base_dir) => base_dir,
                        Err(e) => {
                            error!("failed to create base dir: {:?}", e);
                            return;
                        }
                    };
                    let glb_path = base_dir.join(format!("{:?}_high.glb", msg.agent_id));

                    if !glb_path.exists()
                        && let Err(e) = generate_skinned_mesh(json_path.clone(), glb_path.clone())
                    {
                        error!("mesh generation failed: {:?}", e);
                        return;
                    }

                    avatar_clone.path = Some(glb_path.clone());
                    match sqlite_update_avatar(&db_conn, avatar_clone).await {
                        Ok(_) => {}
                        Err(e) => {
                            error!("Failed to update avatar cache {:?}", e)
                        }
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
                .into_actor(self),
            );
        }
    }
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

impl Handler<HandleNewAvatarAnimation> for Mailbox {
    type Result = ();
    fn handle(&mut self, msg: HandleNewAvatarAnimation, ctx: &mut Self::Context) -> Self::Result {
        let session = match self.session.as_mut() {
            Some(session) => session,
            None => {
                error!("Handle New Avatar Animation failed. Session is not running.");
                return;
            }
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

        let viewer_asset_endpoint = session
            .capability_urls
            .get(&Capability::ViewerAsset)
            .unwrap()
            .to_string();
        let addr = ctx.address();
        let agent_id = avatar.agent_id;
        let sender_id = msg.avatar_animation.sender_id;
        let used_joints = avatar.used_joints.clone();
        let skeleton = avatar.skeleton.clone();
        let mut hasher = DefaultHasher::new();
        used_joints.hash(&mut hasher);
        let joint_hash = format!("{:016x}", hasher.finish());

        let animations = msg.avatar_animation.animations;

        ctx.spawn(
            async move {
                for animation in animations {
                    // if the animation is a default animation, retrieve its JSON from the
                    // default asset path. 
                    let animation_json_path = if let Some(default_animation) =
                        DefaultAnimation::from_uuid(&animation.anim_id)
                    {
                        generated_asset_path()
                            .join("Animations")
                            .join(format!("{}.json", default_animation))
                    } else {
                        // if it is not a default animation, retrieve it from the endpoint and
                        // convert it to an AnimationClip object.
                        match download_asset(
                            ObjectType::Animation.to_string(),
                            animation.anim_id,
                            &viewer_asset_endpoint,
                        )
                        .await
                        {
                            Ok(_) => {
                                //TODO: Implement this
                                warn!("Retrieved animation, but non-default animations are currently unimplemented");
                                return;
                            }
                            Err(e) => {
                                error!(
                                    "failed to retrieve animation {:?}: {:?}",
                                    animation.anim_id, e
                                );
                                return;
                            }
                        }
                    };

                    let filtered_animation_dir =
                        match create_filtered_animation_dir(&animation.anim_id) {
                            Ok(dir) => dir,
                            Err(e) => {
                                warn!(
                                    "Failed to create filtered animation directory for {:?}: {:?}",
                                    animation.anim_id, e
                                );
                                return;
                            }
                        };
                    
                    let filtered_animation_out_path =
                        filtered_animation_dir.join(format!("{}.json", joint_hash));
                    
                    if !filtered_animation_out_path.exists() {
                        if let Err(e) = filter_animation(
                            &animation_json_path,
                            filtered_animation_out_path.clone(),
                            used_joints.clone(),
                        ) {
                            error!(
                                "Failed to filter animation {:?}: {:?}",
                                animation.anim_id, e
                            );
                            return;
                        }
                    }
                    
                    let agent_animation_dir =
                        match create_agent_animation_dir(&agent_id) {
                            Ok(dir) => dir,
                            Err(e) => {
                                warn!(
                                    "Failed to create agent animation directory for {:?}: {:?}",
                                    agent_id, e
                                );
                                return;
                            }
                        };
                    
                    let agent_animation_out_path =
                        agent_animation_dir.join(format!("{}.json", animation.anim_id));
                    
                    if !agent_animation_out_path.exists() {
                        if let Err(e) = apply_joint_scale(
                            &filtered_animation_out_path,
                            agent_animation_out_path.clone(),
                            &skeleton,
                        ) {
                            error!(
                                "Failed to scale animation {:?}: {:?}",
                                animation.anim_id, e
                            );
                            return;
                        }
                    }
                    
                    let animation_out_path =
                        agent_animation_dir.join(format!("{}.glb", animation.anim_id));
                    
                    if let Err(e) = generate_gltf_animation(
                        agent_animation_out_path.clone(),
                        animation_out_path.clone(),
                    ) {
                        error!(
                            "Failed to generate animation {:?}: {:?}",
                            animation.anim_id, e
                        );
                        return;
                    }                   addr.do_send(SendUIMessage {
                       ui_message: UIMessage::new_play_animation(PlayAnimation {
                           player_id: sender_id,
                           animation_path: animation_out_path,
                       }),
                   });
                }
            }
            .into_actor(self),
        );
    }
}

fn effective_parent(
    skeleton: &Skeleton,
    joint_name: JointName,
    joint_filter: &BTreeSet<JointName>,
) -> Option<JointName> {
    let mut parent = skeleton.joints[&joint_name].parent;

    while let Some(parent_name) = parent {
        if joint_filter.contains(&parent_name) {
            return Some(parent_name);
        }

        parent = skeleton.joints[&parent_name].parent;
    }
    None
}

pub fn filter_animation(
    animation_json_path: &PathBuf,
    animation_out_path: PathBuf,
    used_joints: BTreeSet<JointName>,
) -> Result<(), FilterAnimationError> {
    let file =
        fs::File::open(&animation_json_path).map_err(|source| FilterAnimationError::ReadFile {
            path: animation_json_path.clone(),
            source,
        })?;

    let animations: AnimationClip =
        serde_json::from_reader(file).map_err(|source| FilterAnimationError::Deserialize {
            path: animation_json_path.clone(),
            source,
        })?;

    let skeleton: Skeleton = DEFAULT_SKELETON.clone();

    let mut filtered_bind_skeleton = IndexMap::new();

    for (joint_name, joint) in skeleton.joints.iter() {
        if !used_joints.contains(joint_name) {
            continue;
        }

        let joint_global = joint.global_transforms[0].transform;

        let effective_parent = effective_parent(&skeleton, *joint_name, &used_joints);

        let effective_parent_global = match effective_parent {
            Some(parent_name) => skeleton.joints[&parent_name].global_transforms[0].transform,
            None => glam::Mat4::IDENTITY,
        };

        let local = effective_parent_global.inverse() * joint_global;

        let (scale, rotation, translation) = local.to_scale_rotation_translation();

        filtered_bind_skeleton.insert(
            *joint_name,
            BindJoint {
                joint: *joint_name,
                parent: effective_parent,
                translation,
                rotation,
                scale,
            },
        );
    }

    let mut filtered_joints = Vec::new();

    for joint_anim in animations
        .joints
        .iter()
        .filter(|a| used_joints.contains(&a.joint))
    {
        let original_parent_global = match skeleton.joints[&joint_anim.joint].parent {
            Some(parent_name) => skeleton.joints[&parent_name].global_transforms[0].transform,
            None => glam::Mat4::IDENTITY,
        };

        let effective_parent_global =
            match effective_parent(&skeleton, joint_anim.joint, &used_joints) {
                Some(parent_name) => skeleton.joints[&parent_name].global_transforms[0].transform,
                None => glam::Mat4::IDENTITY,
            };

        let parent_conversion = effective_parent_global.inverse() * original_parent_global;

        let mut filtered_animation = joint_anim.clone();

        for keyframe in &mut filtered_animation.translations {
            let animated_local = glam::Mat4::from_translation(keyframe.value);

            let effective_local = parent_conversion * animated_local;

            let (_, _, translation) = effective_local.to_scale_rotation_translation();

            keyframe.value = translation;
        }

        for keyframe in &mut filtered_animation.rotations {
            let animated_local = glam::Mat4::from_quat(keyframe.value);

            let effective_local = parent_conversion * animated_local;

            let (_, rotation, _) = effective_local.to_scale_rotation_translation();

            keyframe.value = rotation;
        }

        for keyframe in &mut filtered_animation.scales {
            let animated_local = glam::Mat4::from_scale(keyframe.value);

            let effective_local = parent_conversion * animated_local;

            let (scale, _, _) = effective_local.to_scale_rotation_translation();

            keyframe.value = scale;
        }

        filtered_joints.push(filtered_animation);
    }

    let animation_clip = AnimationClip {
        bind_skeleton: filtered_bind_skeleton,
        joints: filtered_joints,
    };

    let file = fs::File::create(&animation_out_path).map_err(|source| {
        FilterAnimationError::CreateOutput {
            path: animation_out_path.clone(),
            source,
        }
    })?;

    serde_json::to_writer_pretty(file, &animation_clip).map_err(|source| {
        FilterAnimationError::Serialize {
            path: animation_out_path.clone(),
            source,
        }
    })?;
    Ok(())
}


pub fn apply_joint_scale(
    animation_json_path: &PathBuf,
    animation_out_path: PathBuf,
    target_skeleton: &Skeleton,
) -> Result<(), FilterAnimationError> {
    let file =
        fs::File::open(animation_json_path).map_err(|source| {
            FilterAnimationError::ReadFile {
                path: animation_json_path.clone(),
                source,
            }
        })?;

    let mut animation: AnimationClip =
        serde_json::from_reader(file).map_err(|source| {
            FilterAnimationError::Deserialize {
                path: animation_json_path.clone(),
                source,
            }
        })?;

    // First: scale all local bind translations and animation translations.
    for joint_animation in &mut animation.joints {
        let Some(bind_joint) =
            animation.bind_skeleton.get_mut(&joint_animation.joint)
        else {
            continue;
        };

        let source_bone_length = bind_joint.translation.length();

        let target_bone_length =
            bone_length(target_skeleton, joint_animation.joint, bind_joint.parent);

        if source_bone_length == 0.0 || target_bone_length == 0.0 {
            continue;
        }

        let scale = target_bone_length / source_bone_length;

        bind_joint.translation *= scale;

        for keyframe in &mut joint_animation.translations {
            keyframe.value *= scale;
        }
    }

    let file = fs::File::create(&animation_out_path).map_err(|source| {
        FilterAnimationError::CreateOutput {
            path: animation_out_path.clone(),
            source,
        }
    })?;

    serde_json::to_writer_pretty(file, &animation).map_err(|source| {
        FilterAnimationError::Serialize {
            path: animation_out_path,
            source,
        }
    })?;

    Ok(())
}


fn bone_length(skeleton: &Skeleton, joint_name: JointName, parent_name: Option<JointName>) -> f32 {
    let joint = &skeleton.joints[&joint_name];

    let Some(parent_name) = parent_name else {
        return 1.0;
    };

    let parent = &skeleton.joints[&parent_name];

    let joint_global = joint.global_transforms.last().unwrap().transform;
    let parent_global = parent.global_transforms.last().unwrap().transform;

    let local = parent_global * joint_global.inverse();

    let (_, _, translation) = local.to_scale_rotation_translation();

    translation.length()
}

/// When an object is retrieved in full, the data will be written in serializable json format, to
/// create a cache. The JSON will then be sent to another crate to convert it into a 3d model that
/// can be rendered.
fn write_json<T: Serialize>(data: &T, agent_id: Uuid, filename: &str) -> io::Result<PathBuf> {
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

fn check_skeleton_cycles(skeleton: &Skeleton) -> Result<(), String> {
    fn visit(
        joint: JointName,
        skeleton: &Skeleton,
        visiting: &mut HashSet<JointName>,
        visited: &mut HashSet<JointName>,
    ) -> Result<(), String> {
        if visiting.contains(&joint) {
            return Err(format!("Skeleton cycle detected at {:?}", joint));
        }

        if visited.contains(&joint) {
            return Ok(());
        }

        visiting.insert(joint);

        if let Some(node) = skeleton.joints.get(&joint) {
            for child in &node.children {
                visit(*child, skeleton, visiting, visited)?;
            }
        }

        visiting.remove(&joint);
        visited.insert(joint);

        Ok(())
    }

    let mut visiting = HashSet::new();
    let mut visited = HashSet::new();

    for joint in skeleton.joints.keys() {
        visit(*joint, skeleton, &mut visiting, &mut visited)?;
    }

    Ok(())
}

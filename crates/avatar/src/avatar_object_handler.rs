use benthic_protocol::{
    avatar::{
        skeleton::{Skeleton, SkinJoint},
        types::AvatarObject,
    },
    objects::types::RenderObject,
    session::{
        session::{CacheDir, cache_enabled, create_sub_agent_dir, write_json},
        types::Session,
    },
};
use glam::Mat4;
use indexmap::IndexMap;
use metaverse_mesh::mesh::generate::generate_skinned_mesh;
use std::{collections::BTreeSet, fs, path::PathBuf};
use uuid::Uuid;

use crate::{
    avatar::{Avatar, OutfitObject},
    avatar_object_handler::AvatarState::{FullyLoaded, Incomplete},
    errors::AvatarError,
    skeleton::update_global_avatar_skeleton,
};

/// Enum to describe if the avatar has loaded in or not
pub enum AvatarState {
    /// Avatar is fully loaded. All of the objects required are present.
    FullyLoaded,
    /// Avatar is not fully loaded.
    Incomplete,
}

/// Enum to differentiate type of avatar.
/// Since avatar handling happens very differently between user and non-user updates
pub enum AvatarType {
    /// Avatar is the current user
    User,
    /// Avatar is some other player
    NonUser,
}

/// Add an object to the session's avatar object
///
/// # Parameters
/// - `session`: A mutable reference to the global session
/// - `agent_id`: The uuid of the avatar to add
/// - `object`: The object to add to the session
///
/// # Errors
/// - Returns an [`AvatarError`] on failure
pub fn add_object_to_avatar<C, L, U, I, X>(
    session: &mut Session<C, Avatar, L, U, I, X>,
    agent_id: Uuid,
    object: OutfitObject,
) -> Result<AvatarState, AvatarError> {
    let avatar = session
        .avatars
        .get_mut(&agent_id)
        .ok_or(AvatarError::AgentNotFound { agent: agent_id })?;
    match &object {
        OutfitObject::MeshObject(path) => {
            let file = fs::File::open(path)?;
            let parts: Vec<RenderObject> = serde_json::from_reader(file)?;

            if let Some(skin) = parts.first().and_then(|part| part.skin.as_ref()) {
                update_global_avatar_skeleton(avatar, &skin.skeleton);
            }
        }
        _ => {
            //TODO: unimplemented
            //warn!("Avatars wearing non-mesh objects are currently not supported.")
        }
    }
    avatar.items.push(object);

    if avatar.items.len() == avatar.outfit_size {
        Ok(FullyLoaded)
    } else {
        Ok(Incomplete)
    }
}

/// Method for finalizing an avatar
///
/// This handles generating the full gltf for a player model, and modifying the parenting and global
/// transforms of the skeleton to enable straightforward animation retargeting.
///
/// # Parameters
/// - `agent_id`: ID of the agent to finalize
/// - `skeleton`: The full skeleton of the agent
/// - `used_joints`: List of joints with non-default overrides
/// - `items`: OutfitObjects in the outfit to add to the final mesh
/// - `out_dir`: Path to the avatar's generated dir, usually under ~/local/share/benthic
///
/// # Errors
/// - Returns an [`AvatarError`] on failure
///
/// # Side Effects
/// [Storage: Filesystem]
pub async fn finalize_avatar(
    agent_id: Uuid,
    skeleton: Skeleton,
    used_joints: BTreeSet<SkinJoint>,
    items: Vec<OutfitObject>,
    out_dir: PathBuf,
) -> Result<(PathBuf, Skeleton), AvatarError> {
    let skeleton = finalize_skeleton(skeleton);

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
        global_skeleton: skeleton.clone(),
        used_joints: used_joints.clone(),
    };

    let json_path_str = agent_id.to_string();
    let json_path = PathBuf::from(&json_path_str);

    let json_path = if json_path.exists() && cache_enabled() {
        json_path.clone()
    } else {
        write_json(
            &avatar_object,
            &json_path_str,
            CacheDir::Agent(agent_id),
            &out_dir,
        )?
    };

    let base_dir = create_sub_agent_dir(&out_dir, &agent_id.to_string())?;
    let glb_path = base_dir.join(format!("{:?}_high.glb", agent_id));
    if !glb_path.exists() || !cache_enabled() {
        generate_skinned_mesh(json_path.clone(), glb_path.clone())?
    }

    Ok((glb_path, skeleton))
}

// finalize the skeleton and update the global transforms with its parent joint data
fn finalize_skeleton(mut skeleton: Skeleton) -> Skeleton {
    let globals: IndexMap<SkinJoint, Mat4> = skeleton
        .joints
        .iter()
        .map(|(name, joint)| (*name, joint.global_transforms.last().unwrap().transform))
        .collect();

    for joint in skeleton.joints.values_mut() {
        let global = globals[&joint.name];

        let local = match joint.parent {
            Some(parent_name) => globals[&parent_name].inverse() * global,
            None => global,
        };

        joint.local_transforms.last_mut().unwrap().transform = local;
    }

    skeleton
}

use crate::errors::AvatarError;
use benthic_protocol::{
    avatar::{
        skeleton::{Skeleton, SkinJoint},
        types::AvatarObject,
    },
    session::storage::{CacheDir, cache_enabled, create_sub_agent_dir, write_json},
};
use glam::Mat4;
use indexmap::IndexMap;
use metaverse_mesh::mesh::generate::generate_skinned_mesh;
use std::{
    collections::BTreeSet,
    path::{Path, PathBuf},
};
use uuid::Uuid;

/// Enum to differentiate type of avatar.
/// Since avatar handling happens very differently between user and non-user updates
pub enum AvatarType {
    /// Avatar is the current user
    User,
    /// Avatar is some other player
    NonUser,
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
    items: Vec<PathBuf>,
    out_dir: &Path,
) -> Result<(PathBuf, Skeleton), AvatarError> {
    let skeleton = finalize_skeleton(skeleton);

    let avatar_object = AvatarObject {
        objects: items,
        global_skeleton: skeleton.clone(),
        used_joints,
    };

    let base_dir = create_sub_agent_dir(out_dir, &agent_id.to_string())?;
    let json_path_str = agent_id.to_string();

    let json_path = write_json(
        &avatar_object,
        &json_path_str,
        CacheDir::Agent(agent_id),
        &base_dir,
    )?;

    let glb_path = base_dir.join(format!("{:?}_high.glb", agent_id));

    if !glb_path.exists() || !cache_enabled() {
        generate_skinned_mesh(json_path, glb_path.clone())?;
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

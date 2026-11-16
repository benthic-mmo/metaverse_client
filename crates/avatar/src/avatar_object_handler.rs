use crate::errors::AvatarError;
use benthic_protocol::{
    avatar::{
        skeleton::{Skeleton, SkinJoint},
        types::AvatarObject,
    },
    session::storage::{AgentId, BenthicDirs, cache_enabled},
};
use glam::Mat4;
use indexmap::IndexMap;
use metaverse_mesh::mesh::generate::generate_skinned_mesh;
use std::{collections::BTreeSet, path::PathBuf};
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
    dirs: &BenthicDirs,
) -> Result<(PathBuf, Skeleton), AvatarError> {
    let skeleton = finalize_skeleton(skeleton);

    let avatar_object = AvatarObject {
        objects: items,
        global_skeleton: skeleton.clone(),
        used_joints,
    };

    let agent = AgentId(agent_id);
    dirs.ensure_agent_dir(agent)?;

    let json_path = dirs.agent_avatar_object_json(agent);

    let json = serde_json::to_vec_pretty(&avatar_object)?;
    std::fs::write(&json_path, json)?;

    let gltf_path = dirs.agent_high_gltf(agent);

    if !gltf_path.exists() || !cache_enabled() {
        generate_skinned_mesh(json_path, gltf_path.clone())?;
    }

    Ok((gltf_path, skeleton))
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

use crate::errors::AnimationError;
use benthic_default_asset_converter::generated_animation_path;
use benthic_protocol::{
    animation::default_animations::DefaultAnimation,
    avatar::skeleton::{Skeleton, SkinJoint},
    session::session::{cache_enabled, create_agent_animation_dir, create_filtered_animation_dir},
};
use metaverse_mesh::animation::{
    generate::{
        generate_gltf_animation, retarget_filtered_gltf_animation, retarget_gltf_animation,
    },
    retarget::check_skeleton_cycles,
};
use metaverse_messages::udp::agent::avatar_animation::AnimationEntry;
use std::{
    collections::BTreeSet,
    hash::{DefaultHasher, Hash, Hasher},
    path::PathBuf,
};
use uuid::Uuid;

/// Builds an animation from the AnimationEntry targeting this specific agent's skeleton
///
/// This fully converts animations coming in, filters the joints, and runs a retargeting algorithm to get
/// the animation to play correctly on whatever skeleton configuration the avatar is using.
///
/// # Parameters
/// - `animations`: Vector of [`AnimationEntry`] objects. All of the animations that need to get retargeted
/// - `used_joints`: BTreeSet of joints used by the avatar
/// - `out_dir`: path where aniamtion data should be stored
/// - `agent_id`: agent that this animation is targeting
///
/// # Errors
/// - Returns an [`AnimationError`] on failure
///
/// # Side Effects
/// [Storage: Filesystem ]
pub async fn build_animation(
    animations: Vec<AnimationEntry>,
    used_joints: BTreeSet<SkinJoint>,
    target_skeleton: Skeleton,
    out_dir: PathBuf,
    agent_id: Uuid,
) -> Result<Vec<PathBuf>, AnimationError> {
    let mut hasher = DefaultHasher::new();
    used_joints.hash(&mut hasher);
    let joint_hash = format!("{:016x}", hasher.finish());

    check_skeleton_cycles(&target_skeleton)?;

    let mut animation_paths = Vec::with_capacity(animations.len());

    for animation in animations {
        let animation_json_path = DefaultAnimation::from_uuid(&animation.anim_id)
            .map(|animation| generated_animation_path().join(format!("{animation}.json")))
            .ok_or_else(|| AnimationError::Unimplemented {
                feature: "non-default animations".to_string(),
            })?;

        let agent_animation_dir = create_agent_animation_dir(&out_dir, &agent_id)?;
        let filtered_animation_dir = create_filtered_animation_dir(&out_dir, &animation.anim_id)?;
        let filtered_animation_out_path = filtered_animation_dir.join(format!("{joint_hash}.json"));

        let json_out_path = agent_animation_dir.join(format!("{}.json", animation.anim_id));
        if !filtered_animation_out_path.exists() || !cache_enabled() {
            retarget_gltf_animation(
                &animation_json_path,
                &target_skeleton,
                &filtered_animation_out_path,
                &json_out_path,
            )?;
        } else {
            if !json_out_path.exists() || !cache_enabled() {
                retarget_filtered_gltf_animation(
                    &filtered_animation_out_path,
                    &target_skeleton,
                    &json_out_path,
                )?;
            }
        }

        let animation_out_path = agent_animation_dir.join(format!("{}.glb", animation.anim_id));
        generate_gltf_animation(&json_out_path, &animation_out_path)?;

        animation_paths.push(animation_out_path);
    }
    Ok(animation_paths)
}

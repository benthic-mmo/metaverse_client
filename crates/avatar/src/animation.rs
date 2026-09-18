use std::{
    collections::BTreeSet,
    hash::{DefaultHasher, Hash, Hasher},
    path::PathBuf,
};

use crate::errors::AnimationError;
use benthic_protocol::{
    default_animations::DefaultAnimation,
    session::{cache_enabled, create_filtered_animation_dir},
    skeleton::{JointName, Skeleton},
};
use default_asset_converter::generated_animation_path;
use metaverse_mesh::animation::{
    generate::{
        generate_gltf_animation, retarget_filtered_gltf_animation, retarget_gltf_animation,
    },
    retarget::check_skeleton_cycles,
};
use metaverse_messages::udp::agent::avatar_animation::AnimationEntry;

pub async fn build_animation(
    animations: Vec<AnimationEntry>,
    used_joints: BTreeSet<JointName>,
    target_skeleton: Skeleton,
    out_dir: PathBuf,
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

        let filtered_animation_dir = create_filtered_animation_dir(&animation.anim_id)?;
        let filtered_animation_out_path = filtered_animation_dir.join(format!("{joint_hash}.json"));

        let json_out_path = out_dir.join(format!("{}.json", animation.anim_id));

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

        let animation_out_path = out_dir.join(format!("{}.glb", animation.anim_id));
        generate_gltf_animation(&json_out_path, &animation_out_path)?;

        animation_paths.push(animation_out_path);
    }

    Ok(animation_paths)
}

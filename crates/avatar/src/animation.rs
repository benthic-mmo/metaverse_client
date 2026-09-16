use std::{
    collections::{BTreeSet, HashSet},
    fs,
    hash::{DefaultHasher, Hash, Hasher},
    path::PathBuf,
};

use crate::errors::AnimationError;
use benthic_protocol::{
    default_animations::{AnimationClip, BindJoint, DefaultAnimation},
    session::create_filtered_animation_dir,
    skeleton::{JointName, Skeleton},
};
use default_asset_converter::{generated::DEFAULT_SKELETON, generated_animation_path};
use indexmap::IndexMap;
use metaverse_mesh::animation::generate::generate_gltf_animation;
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

    let mut animation_paths: Vec<PathBuf> = Vec::new();

    for animation in animations {
        let animation_json_path =
            if let Some(default_animation) = DefaultAnimation::from_uuid(&animation.anim_id) {
                generated_animation_path().join(format!("{}.json", default_animation))
            } else {
                // TODO: Implement non-default animations
                return Err(AnimationError::Unimplemented {
                    feature: "non-default animations".to_string(),
                });
            };
        let filtered_animation_dir = create_filtered_animation_dir(&animation.anim_id)?;
        let filtered_animation_out_path =
            filtered_animation_dir.join(format!("{}.json", joint_hash));
        if !filtered_animation_out_path.exists() {
            filter_animation(
                &animation_json_path,
                &filtered_animation_out_path,
                used_joints.clone(),
            )?;
        }

        let json_out_path = out_dir.join(format!("{}.json", animation.anim_id));
        if !json_out_path.exists() {
            apply_joint_scale(
                &filtered_animation_out_path,
                json_out_path.clone(),
                &target_skeleton,
            )?;
        }
        let animation_out_path = out_dir.join(format!("{}.glb", animation.anim_id));
        generate_gltf_animation(&json_out_path, &animation_out_path)?;
        animation_paths.push(animation_out_path);
    }
    Ok(animation_paths)
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
    animation_out_path: &PathBuf,
    used_joints: BTreeSet<JointName>,
) -> Result<(), AnimationError> {
    let file = fs::File::open(&animation_json_path).map_err(|source| AnimationError::ReadFile {
        path: animation_json_path.clone(),
        source,
    })?;

    let animations: AnimationClip =
        serde_json::from_reader(file).map_err(|source| AnimationError::Deserialize {
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

    let file =
        fs::File::create(&animation_out_path).map_err(|source| AnimationError::CreateOutput {
            path: animation_out_path.clone(),
            source,
        })?;

    serde_json::to_writer_pretty(file, &animation_clip).map_err(|source| {
        AnimationError::Serialize {
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
) -> Result<(), AnimationError> {
    let file = fs::File::open(animation_json_path).map_err(|source| AnimationError::ReadFile {
        path: animation_json_path.clone(),
        source,
    })?;

    let mut animation: AnimationClip =
        serde_json::from_reader(file).map_err(|source| AnimationError::Deserialize {
            path: animation_json_path.clone(),
            source,
        })?;

    // First: scale all local bind translations and animation translations.
    for joint_animation in &mut animation.joints {
        let Some(bind_joint) = animation.bind_skeleton.get_mut(&joint_animation.joint) else {
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

    let file =
        fs::File::create(&animation_out_path).map_err(|source| AnimationError::CreateOutput {
            path: animation_out_path.clone(),
            source,
        })?;

    serde_json::to_writer_pretty(file, &animation).map_err(|source| AnimationError::Serialize {
        path: animation_out_path,
        source,
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

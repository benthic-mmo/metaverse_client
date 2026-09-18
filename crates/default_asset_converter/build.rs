use benthic_protocol::default_animations::{
    AnimationClip, DefaultAnimation, JointAnimation, Keyframe,
};
use benthic_protocol::skeleton::JointName::Pelvis;
use benthic_protocol::skeleton::{Joint, JointName, Skeleton, Transform};
use bvh_anim::ChannelType;
use glam::Mat4;
use glam::{Quat, Vec3};
use gltf::Node;
use indexmap::IndexMap;
use std::fs::File;
use std::io::BufReader;
use std::{collections::HashMap, env, fs, path::PathBuf, str::FromStr};
use uuid::Uuid;

fn main() {
    let gen_animations = std::env::var("CARGO_FEATURE_ANIMATIONS").is_ok();
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());

    // copy the default texture to the build dir
    let texture_path = benthic_default_assets::textures();
    fs::copy(
        texture_path.join("default.png"),
        out_dir.join("default.png"),
    )
    .unwrap();

    let animation_path = benthic_default_assets::animations();
    let target_skeleton_path = benthic_default_assets::skeleton().join("skeleton.gltf");

    println!("cargo:rerun-if-changed={}", target_skeleton_path.display());

    if let Ok(entries) = fs::read_dir(&animation_path) {
        for entry in entries.flatten() {
            let path = entry.path();

            if path.is_file() {
                println!("cargo:rerun-if-changed={}", path.display());
            }
        }
    }

    let target_skeleton = skeleton_from_gltf(target_skeleton_path);
    let skeleton_json =
        serde_json::to_string_pretty(&target_skeleton).expect("Failed to serialize skeleton");

    let skeleton_file = out_dir.join("default_skeleton.json");
    fs::write(&skeleton_file, skeleton_json).unwrap();

    println!("cargo:warning=Generating {:?}", skeleton_file);

    if !gen_animations {
        return;
    }

    for entry in fs::read_dir(&animation_path).unwrap() {
        let path = entry.unwrap().path();

        let extension = match path.extension().and_then(|e| e.to_str()) {
            Some(ext @ ("gltf" | "glb" | "bvh")) => ext,
            _ => continue,
        };

        let stem = path.file_stem().unwrap().to_string_lossy();

        let animation_name = match DefaultAnimation::from_str(&stem) {
            Ok(animation) => animation,
            Err(_) => continue,
        };

        #[cfg(feature = "quaternius_adjustments")]
        let axis_conversion = Mat4::from_quat(
            Quat::from_rotation_z(std::f32::consts::FRAC_PI_2)
                * Quat::from_rotation_x(std::f32::consts::FRAC_PI_2),
        );

        #[cfg(not(feature = "quaternius_adjustments"))]
        let axis_conversion = Mat4::IDENTITY;

        // Load animation clip.
        let clip = match extension {
            "gltf" | "glb" => load_gltf_animation(&path, axis_conversion),
            "bvh" => load_bvh_animation(&path),
            _ => unreachable!(),
        };

        let out_path = out_dir.join("Animations");
        fs::create_dir_all(&out_path).unwrap();

        let out_path = out_path.join(format!("{:?}.json", animation_name));

        let json = serde_json::to_string_pretty(&clip).expect("Failed to serialize animation");

        fs::write(&out_path, json).unwrap();

        println!("cargo:warning=Generating animation {:?}", out_path);
    }
}

fn load_gltf_animation(path: &PathBuf, axis_conversion: Mat4) -> AnimationClip {
    let (document, buffers, _) = gltf::import(path).unwrap();

    let mut animation_data: HashMap<JointName, JointAnimation> = HashMap::new();

    let (mut bind_skeleton, mut _root_transform) =
        bind_skeleton_from_gltf(&document, axis_conversion);

    let inverse_conversion = axis_conversion.inverse();

    for anim in document.animations() {
        for channel in anim.channels() {
            let target = channel.target();

            let node_name = match target.node().name() {
                Some(name) => name,
                None => continue,
            };

            let joint_name = match JointName::resolve_joint_name(node_name) {
                Some(name) => name,
                None => continue,
            };

            let reader = channel.reader(|buffer| buffers.get(buffer.index()).map(|d| &d.0[..]));

            let times: Vec<f32> = reader.read_inputs().unwrap().collect();

            let entry = animation_data
                .entry(joint_name)
                .or_insert_with(|| JointAnimation {
                    joint: joint_name,
                    translations: Vec::new(),
                    rotations: Vec::new(),
                    scales: Vec::new(),
                });

            match target.property() {
                gltf::animation::Property::Translation => {
                    if let Some(gltf::animation::util::ReadOutputs::Translations(values)) =
                        reader.read_outputs()
                    {
                        for (time, value) in times.iter().zip(values) {
                            let value = Vec3::from_slice(&value);

                            let converted = axis_conversion
                                * Mat4::from_translation(value)
                                * inverse_conversion;

                            let value = converted.to_scale_rotation_translation().2;

                            if entry.joint != Pelvis {
                                continue;
                            };

                            entry.translations.push(Keyframe { time: *time, value });
                        }
                    }
                }

                gltf::animation::Property::Rotation => {
                    if let Some(gltf::animation::util::ReadOutputs::Rotations(values)) =
                        reader.read_outputs()
                    {
                        for (time, value) in times.iter().zip(values.into_f32()) {
                            let value = Quat::from_array(value);

                            let converted =
                                axis_conversion * Mat4::from_quat(value) * inverse_conversion;

                            let rotation = converted.to_scale_rotation_translation().1;

                            entry.rotations.push(Keyframe {
                                time: *time,
                                value: rotation,
                            });
                        }
                    }
                }

                gltf::animation::Property::Scale => {
                    if let Some(gltf::animation::util::ReadOutputs::Scales(values)) =
                        reader.read_outputs()
                    {
                        for (time, value) in times.iter().zip(values) {
                            let value = Vec3::from_slice(&value);

                            let converted =
                                axis_conversion * Mat4::from_scale(value) * inverse_conversion;

                            let scale = converted.to_scale_rotation_translation().0;

                            entry.scales.push(Keyframe {
                                time: *time,
                                value: scale,
                            });
                        }
                    }
                }

                _ => {}
            }
        }
    }
    #[cfg(feature = "quaternius_adjustments")]
    {
        let hip_rotation = Quat::from_rotation_z(std::f32::consts::PI);
        rotate_hip_subtree(
            &mut bind_skeleton,
            &mut animation_data,
            JointName::HipLeft,
            hip_rotation,
        );

        rotate_hip_subtree(
            &mut bind_skeleton,
            &mut animation_data,
            JointName::HipRight,
            hip_rotation,
        );
    }
    AnimationClip {
        bind_skeleton,
        joints: animation_data.into_values().collect(),
    }
}

fn bind_skeleton_from_gltf(document: &gltf::Document, axis_conversion: Mat4) -> (Skeleton, Mat4) {
    let mut skeleton = Skeleton {
        joints: IndexMap::new(),
        root: vec![JointName::Pelvis],
    };

    let mut root_transform = Mat4::IDENTITY;

    fn recurse(
        node: gltf::Node,
        parent: Option<JointName>,
        skeleton: &mut Skeleton,
        root_transform: &mut Mat4,
        found_root_joint: &mut bool,
    ) {
        let Some(name) = node.name() else {
            return;
        };

        let (translation, rotation, scale) = node.transform().decomposed();

        let local_transform = Mat4::from_scale_rotation_translation(
            Vec3::from(scale),
            Quat::from_array(rotation),
            Vec3::from(translation),
        );

        match JointName::resolve_joint_name(name) {
            Some(joint_name) => {
                *found_root_joint = true;

                skeleton.joints.insert(
                    joint_name,
                    Joint {
                        name: joint_name,
                        parent,
                        children: Vec::new(),
                        local_transforms: vec![Transform {
                            transform: local_transform,
                            id: Uuid::nil(),
                            rank: 0,
                            name: String::new(),
                        }],
                        global_transforms: Vec::new(),
                    },
                );

                if let Some(parent_name) = parent {
                    skeleton
                        .joints
                        .get_mut(&parent_name)
                        .unwrap()
                        .children
                        .push(joint_name);
                }

                for child in node.children() {
                    recurse(
                        child,
                        Some(joint_name),
                        skeleton,
                        root_transform,
                        found_root_joint,
                    );
                }
            }

            None => {
                if !*found_root_joint {
                    *root_transform = local_transform * *root_transform;
                }

                for child in node.children() {
                    recurse(child, parent, skeleton, root_transform, found_root_joint);
                }
            }
        }
    }

    let mut has_parent = std::collections::HashSet::new();

    for node in document.nodes() {
        for child in node.children() {
            has_parent.insert(child.index());
        }
    }

    for node in document.nodes() {
        if !has_parent.contains(&node.index()) {
            let mut found_root_joint = false;

            recurse(
                node,
                None,
                &mut skeleton,
                &mut root_transform,
                &mut found_root_joint,
            );

            if !skeleton.joints.is_empty() {
                break;
            }
        }
    }

    let inverse_conversion = axis_conversion.inverse();

    // Convert the accumulated root transform into Benthic space.
    let converted_root = axis_conversion * root_transform * inverse_conversion;

    // Convert each local joint transform into Benthic space.
    for joint in skeleton.joints.values_mut() {
        let local = joint.local_transforms[0].transform;

        joint.local_transforms[0].transform = axis_conversion * local * inverse_conversion;
    }

    // Bake the pre-skeleton root transform into Pelvis.
    if let Some(pelvis) = skeleton.joints.get_mut(&JointName::Pelvis) {
        let pelvis_local = pelvis.local_transforms[0].transform;

        pelvis.local_transforms[0].transform = converted_root * pelvis_local;
    }

    // Calculate final bind-pose globals from the final locals.
    calculate_bind_globals(&mut skeleton);

    (skeleton, converted_root)
}

fn calculate_bind_globals(skeleton: &mut Skeleton) {
    fn recurse(joint_name: JointName, skeleton: &mut Skeleton, parent_global: Mat4) {
        let local = skeleton.joints[&joint_name].local_transforms[0].transform;

        let global = parent_global * local;

        skeleton
            .joints
            .get_mut(&joint_name)
            .unwrap()
            .global_transforms = vec![Transform {
            transform: global,
            id: Uuid::nil(),
            rank: 0,
            name: String::new(),
        }];

        let children = skeleton.joints[&joint_name].children.clone();

        for child in children {
            recurse(child, skeleton, global);
        }
    }

    let roots: Vec<JointName> = skeleton
        .joints
        .values()
        .filter(|joint| joint.parent.is_none())
        .map(|joint| joint.name)
        .collect();

    for root in roots {
        recurse(root, skeleton, Mat4::IDENTITY);
    }
}
fn load_bvh_animation(path: &PathBuf) -> AnimationClip {
    let bvh_file = File::open(path).unwrap();

    let bvh = bvh_anim::from_reader(BufReader::new(bvh_file)).unwrap();

    let mut animation_data: HashMap<JointName, JointAnimation> = HashMap::new();

    let axis_correction = Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2);

    for (frame_idx, frame) in bvh.frames().enumerate() {
        let frame_time = bvh.frame_time().as_secs_f32();

        let time = frame_idx as f32 * frame_time;

        for joint in bvh.joints() {
            let joint_name =
                match JointName::resolve_joint_name(joint.data().name().to_str().unwrap()) {
                    Some(j) => j,
                    None => continue,
                };

            let mut translation = Vec3::ZERO;
            let mut rotation = Quat::IDENTITY;

            for channel in joint.data().channels() {
                let value = frame.get(channel).unwrap();

                let radians = value.to_radians();

                match channel.channel_type() {
                    ChannelType::PositionX => translation.x = *value,
                    ChannelType::PositionY => translation.y = *value,
                    ChannelType::PositionZ => translation.z = *value,

                    ChannelType::RotationX => rotation *= Quat::from_rotation_x(radians),

                    ChannelType::RotationY => rotation *= Quat::from_rotation_y(radians),

                    ChannelType::RotationZ => rotation *= Quat::from_rotation_z(radians),
                }
            }

            let corrected_translation = (axis_correction * translation) * 0.01;

            let corrected_rotation = match joint_name {
                JointName::Pelvis => rotation,
                _ => axis_correction * rotation * axis_correction.inverse(),
            };

            let entry = animation_data
                .entry(joint_name)
                .or_insert_with(|| JointAnimation {
                    joint: joint_name,
                    translations: Vec::new(),
                    rotations: Vec::new(),
                    scales: Vec::new(),
                });

            if joint_name == JointName::Pelvis {
                entry.translations.push(Keyframe {
                    time,
                    value: corrected_translation,
                });
            }

            entry.rotations.push(Keyframe {
                time,
                value: corrected_rotation,
            });

            entry.scales.push(Keyframe {
                time,
                value: Vec3::ONE,
            });
        }
    }

    AnimationClip {
        bind_skeleton: Skeleton::default(),
        joints: animation_data.into_values().collect(),
    }
}

/// This is used to generate the default skeleton from the GLTF file.
///
/// The GLTF has an incorrect axis basis. I'm keeping it as gltf for
/// viewability and editability, but parsing it to JSON must convert
/// to the OpenSim orientation.
fn skeleton_from_gltf(skeleton_path: PathBuf) -> Skeleton {
    let (document, _, _) = gltf::import(&skeleton_path)
        .unwrap_or_else(|_| panic!("Failed to load skeleton {:?}", skeleton_path));

    let nodes: Vec<Node> = document.nodes().collect();

    let mut joints = IndexMap::new();

    let pelvis = nodes
        .iter()
        .find(|node| node.name() == Some("mPelvis"))
        .unwrap_or_else(|| panic!("Failed to find mPelvis in skeleton {:?}", skeleton_path));

    build_joint_recursive(pelvis.index(), None, &nodes, Mat4::IDENTITY, &mut joints);

    // THE AXIS CONVERSION HAPPENS HERE
    let axis_conversion = Mat4::from_quat(Quat::from_rotation_x(std::f32::consts::FRAC_PI_2));

    let inverse_conversion = axis_conversion.inverse();

    for joint in joints.values_mut() {
        for transform in &mut joint.global_transforms {
            transform.transform = axis_conversion * transform.transform * inverse_conversion;
        }

        for transform in &mut joint.local_transforms {
            transform.transform = axis_conversion * transform.transform * inverse_conversion;
        }
    }

    Skeleton {
        joints,
        root: vec![JointName::Pelvis],
    }
}

fn build_joint_recursive(
    index: usize,
    parent: Option<JointName>,
    nodes: &[Node],
    parent_global: Mat4,
    joints: &mut IndexMap<JointName, Joint>,
) {
    let node = nodes[index].clone();

    let name = JointName::from_str(node.name().unwrap()).unwrap();

    if joints.contains_key(&name) {
        return;
    }

    let (translation, rotation, scale) = node.transform().decomposed();

    let local = Mat4::from_scale_rotation_translation(
        Vec3::from(scale),
        Quat::from_array(rotation),
        Vec3::from(translation),
    );

    let global = parent_global * local;

    let mut children = Vec::new();

    for child in node.children() {
        let child_name = JointName::from_str(child.name().unwrap())
            .unwrap_or_else(|err| panic!("errored on {:?}, {:?}", child.name(), err));

        children.push(child_name);

        build_joint_recursive(child.index(), Some(name), nodes, global, joints);
    }

    joints.insert(
        name,
        Joint {
            name,
            parent,
            children,
            global_transforms: vec![Transform {
                name: "Default".to_string(),
                id: Uuid::nil(),
                transform: global,
                rank: 0,
            }],
            local_transforms: vec![Transform {
                name: "Default".to_string(),
                id: Uuid::nil(),
                transform: local,
                rank: 0,
            }],
        },
    );
}

fn rotate_hip_subtree(
    skeleton: &mut Skeleton,
    animation_data: &mut HashMap<JointName, JointAnimation>,
    hip: JointName,
    rotation: Quat,
) {
    let correction = Mat4::from_quat(rotation);

    let joints = match hip {
        JointName::HipLeft => [JointName::HipLeft, JointName::KneeLeft, JointName::FootLeft],
        JointName::HipRight => [
            JointName::HipRight,
            JointName::KneeRight,
            JointName::FootRight,
        ],
        _ => return,
    };

    // Rotate hip.
    {
        let joint_name = joints[0];

        let local = skeleton.joints[&joint_name].local_transforms[0].transform;
        skeleton
            .joints
            .get_mut(&joint_name)
            .unwrap()
            .local_transforms[0]
            .transform = local * correction;

        if let Some(animation) = animation_data.get_mut(&joint_name) {
            for keyframe in &mut animation.rotations {
                let corrected = Mat4::from_quat(keyframe.value) * correction;
                keyframe.value = corrected.to_scale_rotation_translation().1;
            }
        }
    }

    // Recalculate globals: hip -> knee -> foot.
    let parent_global = skeleton.joints[&hip]
        .parent
        .map(|parent| {
            skeleton.joints[&parent]
                .global_transforms
                .last()
                .unwrap()
                .transform
        })
        .unwrap_or(Mat4::IDENTITY);

    let hip_local = skeleton.joints[&joints[0]]
        .local_transforms
        .last()
        .unwrap()
        .transform;
    let hip_global = parent_global * hip_local;

    skeleton
        .joints
        .get_mut(&joints[0])
        .unwrap()
        .global_transforms
        .last_mut()
        .unwrap()
        .transform = hip_global;

    let knee_local = skeleton.joints[&joints[1]]
        .local_transforms
        .last()
        .unwrap()
        .transform;
    let knee_global = hip_global * knee_local;

    skeleton
        .joints
        .get_mut(&joints[1])
        .unwrap()
        .global_transforms
        .last_mut()
        .unwrap()
        .transform = knee_global;

    let foot_local = skeleton.joints[&joints[2]]
        .local_transforms
        .last_mut()
        .unwrap()
        .transform;
    let foot_global = knee_global * foot_local;

    skeleton
        .joints
        .get_mut(&joints[2])
        .unwrap()
        .global_transforms
        .last_mut()
        .unwrap()
        .transform = foot_global;
}

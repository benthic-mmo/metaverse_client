use crate::avatar::Avatar;
use glam::{Mat4, Vec4};
use indexmap::IndexMap;
use metaverse_messages::{
    http::scene::SceneObject,
    utils::skeleton::{Joint, JointName, Skeleton, Transform},
};
use uuid::Uuid;

/// This function takes an object's skeleton, and applies it to the agent's combined skeleton. The
/// combined skeleton contains all of the transforms for all of the joints, ranked by how many
/// elements of the outfit contain the same transform.
/// Highest ranking joint values will always be last.
pub fn update_global_avatar_skeleton(avatar: &mut Avatar, skeleton: &Skeleton) {
    for joint in skeleton.joints.values() {
        if let Some(global_joint) = avatar.skeleton.joints.get_mut(&joint.name) {
            for transform in &joint.transforms {
                set_rank(global_joint, transform, |j| &mut j.transforms);
            }
            for local_transform in &joint.local_transforms {
                set_rank(global_joint, local_transform, |j| &mut j.local_transforms);
            }
        }
    }
}

/// Determine the local and global joint transforms for a skinned SceneObject.
pub fn create_skeleton(scene_root: SceneObject) -> Option<Skeleton> {
    // if the object has a mesh, handle the skeleton
    if let Some(mesh) = &scene_root.sculpt.mesh {
        // fetch the default skeleton which is generated at compile time
        // this will be used for calculating transforms, and getting joint local transforms for the
        // root object.
        let default_skeleton = include!(concat!(env!("OUT_DIR"), "/default_skeleton.rs"));

        let mut joints = IndexMap::new();
        // for every joint in the skin
        for (i, name) in mesh.skin.as_ref().unwrap().joint_names.iter().enumerate() {
            // apply the rotations from the default skeleton to the object
            // the default skeleton's transforms are stored in [0]
            // these rotations need to be applied, because the IBMs from -> *mut c_charthe server are mostly
            // the identity matrix, and only contain translation information. The default
            // skeleton contains the rotations.
            let default_joints = default_skeleton.joints.get(name).unwrap().clone();
            let mut default_transform = default_joints.transforms[0].transform;

            default_transform.w_axis = Vec4::new(0.0, 0.0, 0.0, 1.0);
            let transform_matrix =
                default_transform * mesh.skin.as_ref().unwrap().inverse_bind_matrices[i];

            let transform = Transform {
                name: scene_root.name.clone(),
                id: scene_root.sculpt.texture,
                transform: transform_matrix,
                // Default transforms are rank 0, and will stay at position 0 in the array.
                // Custom transforms start at rank 1.
                rank: 1,
            };

            // create the joint object that contains the calculted transforms
            let joint = Joint {
                name: *name,
                parent: default_joints.parent,
                children: default_joints.children.into_iter().collect(),
                transforms: vec![transform.clone()],
                // leave the local tranforms empty for now. They will be added after the loop.
                local_transforms: vec![],
            };
            joints.insert(*name, joint.clone());
        }

        // determine the root joints of the skeleton
        let root_joints: Vec<JointName> = joints
            .values()
            .filter(|joint| joint.parent.is_none())
            .map(|joint| joint.name)
            .collect();

        // Attach the IBMs to their corresponding joint name
        let ibm_map: IndexMap<JointName, Mat4> = mesh
            .skin
            .as_ref()
            .unwrap()
            .joint_names
            .iter()
            .cloned()
            .enumerate()
            .map(|(i, name)| (name, mesh.skin.as_ref().unwrap().inverse_bind_matrices[i]))
            .collect();

        let last_transforms: IndexMap<JointName, Mat4> = joints
            .iter()
            .map(|(name, joint)| (*name, joint.transforms.last().unwrap().transform))
            .collect();

        for (_, joint) in joints.iter_mut() {
            if let Some(parent_name) = &joint.parent {
                // if last_transforms doesn't contain the parent, it means that is the root, and
                // the parent should be calculated via the default skeleton.
                let parent = last_transforms
                    .get(parent_name)
                    .or_else(|| {
                        Some(
                            &default_skeleton.joints.get(parent_name).unwrap().transforms[0]
                                .transform,
                        )
                    })
                    .unwrap();

                let child = last_transforms.get(&joint.name).unwrap();
                // IBM-based local matrix
                let local_matrix = *parent * child.inverse();
                let local_transform = Transform {
                    name: scene_root.name.clone(),
                    id: scene_root.sculpt.texture,
                    transform: local_matrix,
                    rank: 1,
                };
                joint.local_transforms.push(local_transform);
            } else {
                // root joint: local transform = its own IBM
                if let Some(child_ibm) = ibm_map.get(&joint.name) {
                    let local_transform = Transform {
                        name: scene_root.name.clone(),
                        id: scene_root.sculpt.texture,
                        transform: *child_ibm,
                        rank: 1,
                    };
                    joint.local_transforms.push(local_transform);
                }
            }
        }
        Some(Skeleton {
            root: root_joints,
            joints,
        })
    } else {
        None
    }
}

// Modify the rank for local and global transforms.
fn set_rank<F>(joint: &mut Joint, transform: &Transform, mut get_vec: F)
where
    F: FnMut(&mut Joint) -> &mut Vec<Transform>,
{
    let transforms = get_vec(joint);

    // Track whether we matched only rank 0
    let mut matched_rank0 = false;

    for t in transforms.iter_mut().skip(1) {
        if t.transform.abs_diff_eq(transform.transform, 1e-4) {
            t.rank += 1;
            transforms.sort_by(|a, b| a.rank.cmp(&b.rank));
            return;
        }
    }

    // If we didn’t find a non-zero match, check rank 0.
    if let Some(first) = transforms.first()
        && first.transform.abs_diff_eq(transform.transform, 1e-4) {
            matched_rank0 = true;
        }

    // If it matched only rank 0 or none at all, create a new transform
    if matched_rank0
        || !transforms
            .iter()
            .any(|t| t.transform.abs_diff_eq(transform.transform, 1e-4))
    {
        let mut new_t = transform.clone();
        // If it matched only rank 0, start at rank 1
        if matched_rank0 {
            new_t.rank = 1;
        }
        transforms.push(new_t);
        transforms.sort_by(|a, b| a.rank.cmp(&b.rank));
    }
}

use crate::avatar::Avatar;
use glam::{Mat4, Vec4};
use indexmap::IndexMap;
use metaverse_messages::{
    capabilities::scene::SceneObject,
    utils::skeleton::{Joint, JointName, Skeleton, Transform},
};
use std::{
    collections::{HashMap, HashSet},
    sync::{Arc, Mutex},
};
use uuid::Uuid;

pub fn create_skeleton(
    scene_root: SceneObject,
    agent_list: Arc<Mutex<HashMap<Uuid, Avatar>>>,
    agent_id: Uuid,
) -> Option<Skeleton> {
    if let Some(mesh) = &scene_root.sculpt.mesh {
        let mut joints = IndexMap::new();
        let valid_names: HashSet<_> = mesh.skin.joint_names.iter().cloned().collect();

        for (i, name) in mesh.skin.joint_names.iter().enumerate() {
            if let Some(agent) = agent_list.lock().unwrap().get_mut(&agent_id) {
                let default_joints = agent.skeleton.joints.get(name).unwrap().clone();

                // apply the rotations from the default skeleton to the object
                // the default skeleton's transforms are stored in [0]
                let mut default_transform = default_joints.transforms[0].transform.clone();
                default_transform.w_axis = Vec4::new(0.0, 0.0, 0.0, 1.0);
                let transform_matrix = default_transform * mesh.skin.inverse_bind_matrices[i];
                let transform = Transform {
                    name: scene_root.name.clone(),
                    id: scene_root.sculpt.texture,
                    transform: transform_matrix,
                    rank: 1,
                };

                // create the joint object that contains the calculted transforms
                let joint = Joint {
                    name: name.clone(),
                    parent: default_joints.parent.filter(|p| valid_names.contains(p)),
                    children: default_joints
                        .children
                        .into_iter()
                        .filter(|p| valid_names.contains(p))
                        .collect(),
                    transforms: vec![transform.clone()],
                    // leave the local tranforms empty for now. They will be added after the loop.
                    local_transforms: vec![],
                };
                joints.insert(*name, joint.clone());

                // update the global skeleton with the transforms
                if let Some(joint) = agent.skeleton.joints.get_mut(name) {
                    // if a transform is specified by multiple parts of an outfit, increase the
                    // rank of it in the skeleton.
                    if let Some(existing) = joint
                        .transforms
                        .iter_mut()
                        .find(|t| t.transform.abs_diff_eq(transform.transform, 1e-4))
                    {
                        existing.rank += 1;
                        // Then sort the transforms array to place that transform last.
                        joint.transforms.sort_by(|a, b| a.rank.cmp(&b.rank));
                    } else {
                        joint.transforms.push(transform);
                    }
                }
            };
        }
        // create a skeleton that contains the root nodes
        let root_joints: Vec<JointName> = joints
            .values()
            .filter(|joint| joint.parent.is_none())
            .map(|joint| joint.name.clone())
            .collect();

        // create the local transforms
        let ibm_map: IndexMap<JointName, Mat4> = mesh
            .skin
            .joint_names
            .iter()
            .cloned()
            .enumerate()
            .map(|(i, name)| (name, mesh.skin.inverse_bind_matrices[i]))
            .collect();

        // create the local transforms
        if let Some(agent) = agent_list.lock().unwrap().get_mut(&agent_id) {
            // the highest ranking joint will always be last. Retrieve the name and highest ranked
            // transform to calculate the local joint transforms
            let calculated_transforms: IndexMap<JointName, Mat4> = agent
                .skeleton
                .joints
                .iter()
                .map(|(name, joint)| (name.clone(), joint.transforms.last().unwrap().transform))
                .collect();
            for joint in agent.skeleton.joints.values_mut() {
                if let Some(parent_name) = &joint.parent {
                    let parent = calculated_transforms.get(parent_name).unwrap().clone();
                    let child = calculated_transforms.get(&joint.name).unwrap().clone();
                    // IBM-based local matrix
                    let local_matrix = parent * child.inverse();
                    let local_transform = Transform {
                        name: joint.name.to_string().clone(),
                        id: scene_root.sculpt.texture,
                        transform: local_matrix,
                        rank: 1,
                    };
                    joint.local_transforms.push(local_transform);
                } else {
                    // root joint: local transform = its own IBM
                    if let Some(child_ibm) = ibm_map.get(&joint.name) {
                        let local_transform = Transform {
                            name: joint.name.to_string().clone(),
                            id: scene_root.sculpt.texture,
                            transform: *child_ibm,
                            rank: 1,
                        };
                        joint.local_transforms.push(local_transform);
                    }
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

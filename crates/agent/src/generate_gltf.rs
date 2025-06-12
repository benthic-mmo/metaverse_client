use glam::{usize, Vec3};
use gltf_json::{
    accessor::{ComponentType, GenericComponentType},
    buffer::{Stride, Target, View},
    mesh::{Mode, Primitive, Semantic},
    scene::UnitQuaternion,
    validation::{
        Checked::{self, Valid},
        USize64,
    },
    Accessor, Index, Mesh, Node, Root, Scene, Skin, Value,
};
use metaverse_messages::utils::skeleton::JointName;
use metaverse_messages::utils::skeleton::Skeleton;
use rgb::bytemuck;
use std::{
    borrow::Cow,
    collections::{BTreeMap, BTreeSet, HashMap},
    fs::File,
    mem,
    path::PathBuf,
    vec,
};

use crate::avatar::OutfitObject;

pub fn generate_baked_avatar(
    outfit_objects: Vec<OutfitObject>,
    skeleton: Skeleton,
    path: PathBuf,
) -> Result<PathBuf, Box<dyn std::error::Error>> {
    let mut root = Root::default();
    let mut combined_buffer: Vec<u8> = Vec::new();
    let mut bones: BTreeSet<JointName> = BTreeSet::new();
    let mut combined_weights = Vec::new();
    let mut nodes = Vec::new();

    let buffer = root.push(gltf_json::Buffer {
        byte_length: USize64::from(0_usize),
        extensions: Default::default(),
        extras: Default::default(),
        name: Some("Combined Avatar".to_string()),
        uri: None,
    });
    for object in outfit_objects.clone() {
        if let OutfitObject::RiggedObject(rigged) = object {
            for (name, _joint) in &rigged.skeleton.joints {
                bones.insert(name.clone());
            }
        }
    }

    for object in outfit_objects.clone() {
        if let OutfitObject::RiggedObject(rigged) = object {
            for secene_object in rigged.scene_group.parts {
                if let Some(mesh) = secene_object.sculpt.mesh.as_ref() {
                    let vertices_transformed: Vec<Vec3> =
                        mesh.high_level_of_detail.vertices.clone();
                    let (min, max) = bounding_coords(&vertices_transformed);

                    let padded_vertex_bytes = to_padded_byte_vector(&vertices_transformed);
                    while combined_buffer.len() % 4 != 0 {
                        combined_buffer.push(0);
                    }
                    let vertex_offset = combined_buffer.len();
                    let buffer_view = root.push(View {
                        buffer,
                        byte_length: USize64::from(padded_vertex_bytes.len()),
                        byte_offset: Some(USize64::from(vertex_offset)),
                        byte_stride: Some(Stride(mem::size_of::<Vec3>())),
                        target: Some(Valid(Target::ArrayBuffer)),
                        extensions: Default::default(),
                        extras: Default::default(),
                        name: None,
                    });
                    combined_buffer.extend_from_slice(&padded_vertex_bytes);
                    let positions_accessor = root.push(Accessor {
                        buffer_view: Some(buffer_view),
                        byte_offset: Some(USize64(0)),
                        count: USize64::from(mesh.high_level_of_detail.vertices.len()),
                        component_type: Valid(GenericComponentType(ComponentType::F32)),
                        type_: Valid(gltf_json::accessor::Type::Vec3),
                        min: Some(Value::from(Vec::from(min))),
                        max: Some(Value::from(Vec::from(max))),
                        normalized: false,
                        sparse: None,
                        name: None,
                        extensions: Default::default(),
                        extras: Default::default(),
                    });
                    while combined_buffer.len() % 2 != 0 {
                        combined_buffer.push(0);
                    }

                    let mut index_bytes =
                        Vec::with_capacity(mesh.high_level_of_detail.indices.len() * 2);
                    for index in &mesh.high_level_of_detail.indices {
                        index_bytes.extend_from_slice(&index.to_le_bytes());
                    }
                    let index_view = root.push(View {
                        buffer,
                        byte_length: USize64::from(index_bytes.len()),
                        byte_offset: Some(USize64::from(combined_buffer.len())),
                        byte_stride: None,
                        target: Some(Valid(Target::ElementArrayBuffer)),
                        extensions: Default::default(),
                        extras: Default::default(),
                        name: None,
                    });
                    combined_buffer.extend_from_slice(&index_bytes);
                    while combined_buffer.len() % 4 != 0 {
                        combined_buffer.push(0);
                    }

                    let index_accessor = root.push(Accessor {
                        buffer_view: Some(index_view),
                        byte_offset: Some(USize64(0)),
                        count: USize64::from(mesh.high_level_of_detail.indices.len()),
                        component_type: Valid(GenericComponentType(ComponentType::U16)),
                        type_: Valid(gltf_json::accessor::Type::Scalar),
                        normalized: false,
                        sparse: None,
                        name: None,
                        extensions: Default::default(),
                        extras: Default::default(),
                        min: None,
                        max: None,
                    });

                    let mut joint_indices_bytes = Vec::new();
                    let mut joint_weights_bytes = Vec::new();
                    for vw in &mesh.high_level_of_detail.weights {
                        let joints: Vec<u8> = vw
                            .joint_name
                            .iter()
                            .filter_map(|j| {
                                bones.iter().enumerate().find_map(|(i, joint_name)| {
                                    if joint_name == j {
                                        Some(i as u8)
                                    } else {
                                        None
                                    }
                                })
                            })
                            .collect();
                        for (&joint, &weight) in joints.iter().zip(vw.weights.iter()) {
                            joint_indices_bytes.extend_from_slice(&joint.to_le_bytes());
                            joint_weights_bytes.extend_from_slice(&weight.to_le_bytes());
                        }
                    }
                    for vw in &mesh.high_level_of_detail.weights {
                        // resolve joint indices from joint names
                        let joints: Vec<u8> = vw
                            .joint_name
                            .iter()
                            .filter_map(|j| {
                                bones.iter().enumerate().find_map(|(i, joint_name)| {
                                    if joint_name == j {
                                        Some(i as u8)
                                    } else {
                                        None
                                    }
                                })
                            })
                            .collect();

                        let indices: [u8; 4] = joints.try_into().unwrap_or([0, 0, 0, 0]);

                        for (&joint, &weight) in indices.iter().zip(vw.weights.iter()) {
                            joint_indices_bytes.extend_from_slice(&joint.to_le_bytes());
                            joint_weights_bytes.extend_from_slice(&weight.to_le_bytes());
                        }
                    }

                    // --- joint indices view/accessor
                    let joint_indices_view = root.push(View {
                        buffer,
                        byte_length: USize64::from(joint_indices_bytes.len()),
                        byte_offset: Some(USize64::from(combined_buffer.len())),
                        byte_stride: Some(Stride(4 * std::mem::size_of::<u8>())),
                        target: Some(Valid(Target::ArrayBuffer)),
                        extensions: None,
                        extras: Default::default(),
                        name: None,
                    });
                    combined_buffer.extend_from_slice(&joint_indices_bytes);
                    let joint_indices_accessor = root.push(Accessor {
                        buffer_view: Some(joint_indices_view),
                        byte_offset: Some(USize64(0)),
                        count: USize64::from(mesh.high_level_of_detail.weights.len()),
                        component_type: Valid(GenericComponentType(ComponentType::U8)),
                        type_: Valid(gltf_json::accessor::Type::Vec4),
                        normalized: false,
                        min: None,
                        max: None,
                        name: None,
                        sparse: None,
                        extensions: None,
                        extras: Default::default(),
                    });
                    // --- joint weights view/accessor
                    let joint_weights_view = root.push(View {
                        buffer,
                        byte_length: USize64::from(joint_weights_bytes.len()),
                        byte_offset: Some(USize64::from(combined_buffer.len())),
                        byte_stride: Some(Stride(4 * std::mem::size_of::<f32>())),
                        target: Some(Valid(Target::ArrayBuffer)),
                        extensions: None,
                        extras: Default::default(),
                        name: None,
                    });
                    combined_buffer.extend_from_slice(&joint_weights_bytes);
                    let joint_weights_accessor = root.push(Accessor {
                        buffer_view: Some(joint_weights_view),
                        byte_offset: Some(USize64(0)),
                        count: USize64::from(mesh.high_level_of_detail.weights.len()),
                        component_type: Valid(GenericComponentType(ComponentType::F32)),
                        type_: Valid(gltf_json::accessor::Type::Vec4),
                        normalized: false,
                        min: None,
                        max: None,
                        name: None,
                        sparse: None,
                        extensions: None,
                        extras: Default::default(),
                    });

                    let mut attributes = BTreeMap::new();
                    attributes.insert(Valid(Semantic::Positions), positions_accessor);
                    attributes.insert(Valid(Semantic::Joints(0)), joint_indices_accessor);
                    attributes.insert(Valid(Semantic::Weights(0)), joint_weights_accessor);

                    let primitive = Primitive {
                        attributes,
                        indices: Some(index_accessor),
                        material: None,
                        mode: Valid(Mode::Triangles),
                        targets: None,
                        extensions: Default::default(),
                        extras: Default::default(),
                    };

                    let mesh_index = root.push(Mesh {
                        primitives: vec![primitive],
                        weights: None,
                        extensions: Default::default(),
                        extras: Default::default(),
                        name: None,
                    });

                    let node_index = root.push(Node {
                        mesh: Some(mesh_index),
                        skin: None,
                        children: None,
                        name: Some(secene_object.name.to_string()),
                        ..Default::default()
                    });
                    nodes.push(node_index);
                }
                combined_weights.push(
                    secene_object
                        .sculpt
                        .mesh
                        .unwrap()
                        .high_level_of_detail
                        .weights,
                );
            }
        }
    }

    let mut joint_to_node: HashMap<JointName, Index<Node>> = HashMap::new();
    let mut skeleton_nodes = Vec::new();
    let mut ibm_matrices: Vec<[f32; 16]> = Vec::new();

    for joint_name in &bones {
        if let Some(joint) = skeleton.joints.get(joint_name) {
            let (scale, rotation, translation) = joint
                .local_transforms
                .last()
                .unwrap()
                .transform
                .to_scale_rotation_translation();
            let node_index = root.push(Node {
                name: Some(joint_name.to_string()),
                scale: Some(scale.into()),
                rotation: Some(UnitQuaternion([
                    rotation.x, rotation.y, rotation.z, rotation.w,
                ])),
                translation: Some(translation.into()),
                ..Default::default()
            });
            if skeleton.root.contains(joint_name) {
                nodes.push(node_index)
            }

            joint_to_node.insert(joint_name.clone(), node_index);
            skeleton_nodes.push(node_index);
            ibm_matrices.push(joint.transforms.last().unwrap().transform.to_cols_array());
        };
    }
    // now that we know the locations of all of the joints in the array, add the children
    for joint_name in &bones {
        let joint = skeleton.joints.get(joint_name).unwrap();
        if let Some(parent_name) = joint.parent {
            let parent_index = joint_to_node[&parent_name];
            let child_index = joint_to_node[joint_name];
            root.nodes[parent_index.value()]
                .children
                .get_or_insert_with(Vec::new)
                .push(Index::new(child_index.value() as u32));
        }
    }

    while combined_buffer.len() % 4 != 0 {
        combined_buffer.push(0);
    }

    let ibm_byte_offset = combined_buffer.len();
    for mat in &ibm_matrices {
        for f in mat.iter() {
            combined_buffer.extend_from_slice(&f.to_le_bytes());
        }
    }

    let ibm_byte_length = ibm_matrices.len() * 16 * std::mem::size_of::<f32>();
    let ibm_view = View {
        buffer: Index::new(0),
        byte_length: USize64::from(ibm_byte_length as usize),
        byte_offset: Some(USize64(ibm_byte_offset as u64)),
        byte_stride: None,
        target: None,
        name: Some("inverse_bind_matrices_view".to_string()),
        extensions: Default::default(),
        extras: Default::default(),
    };
    let ibm_view_index = root.push(ibm_view);

    // create accessor of type Mat4
    let ibm_accessor = Accessor {
        sparse: None,
        buffer_view: Some(ibm_view_index),
        byte_offset: Some(USize64(0)),
        count: USize64::from(ibm_matrices.len()),
        component_type: Checked::Valid(gltf_json::accessor::GenericComponentType(
            ComponentType::F32,
        )),
        extensions: Default::default(),
        extras: Default::default(),
        max: None,
        min: None,
        name: Some("inverse_bind_matrices_accessor".to_string()),
        normalized: false,
        type_: Checked::Valid(gltf_json::accessor::Type::Mat4),
    };
    let ibm_accessor_index = root.push(ibm_accessor);

    // create skin referencing the joint nodes and the ibm accessor
    let skin = Skin {
        joints: skeleton_nodes.clone(),
        inverse_bind_matrices: Some(ibm_accessor_index),
        skeleton: skeleton_nodes.get(0).cloned(),
        extensions: Default::default(),
        extras: Default::default(),
        name: None,
    };
    let skin_index = root.push(skin);
    for node in root.nodes.iter_mut() {
        if node.mesh.is_some() {
            node.skin = Some(skin_index)
        }
    }
    let rotated_root_index = Index::new(root.nodes.len() as u32);
    root.nodes.push(Node {
        camera: None,
        children: Some(nodes.clone()),
        name: Some("RotatedRoot".to_string()),
        ..Default::default()
    });

    root.push(Scene {
        extensions: Default::default(),
        extras: Default::default(),
        name: Some("asdf".to_string()),
        nodes: vec![rotated_root_index],
    });

    root.buffers[buffer.value() as usize].byte_length = USize64::from(combined_buffer.len());

    let json_string = gltf_json::serialize::to_string(&root)?;
    let glb = gltf::binary::Glb {
        header: gltf::binary::Header {
            magic: *b"glTF",
            version: 2,
            length: (json_string.len() + combined_buffer.len()).try_into()?,
        },
        json: Cow::Owned(json_string.into_bytes()),
        bin: Some(Cow::Owned(combined_buffer)),
    };
    glb.to_writer(File::create(&path)?)?;
    Ok(path)
}

/// Converts a byte vector to a vector aligned to a mutiple of 4
fn to_padded_byte_vector(data: &[Vec3]) -> Vec<u8> {
    let flat: Vec<[f32; 3]> = data.iter().map(|v| [v.x, v.y, v.z]).collect();
    let byte_slice: &[u8] = bytemuck::cast_slice(&flat);
    let mut new_vec: Vec<u8> = byte_slice.to_owned();

    while new_vec.len() % 4 != 0 {
        new_vec.push(0); // pad to multiple of four bytes
    }

    new_vec
}

/// determines the highest and lowest points on the mesh to store as min and max
///fn bounding_coords(points: &[Vec3]) -> ([f32; 3], [f32; 3]) {
fn bounding_coords(points: &[Vec3]) -> ([f32; 3], [f32; 3]) {
    let mut min = [f32::MAX, f32::MAX, f32::MAX];
    let mut max = [f32::MIN, f32::MIN, f32::MIN];

    for p in points {
        for i in 0..3 {
            min[i] = f32::min(min[i], p[i]);
            max[i] = f32::max(max[i], p[i]);
        }
    }
    (min, max)
}

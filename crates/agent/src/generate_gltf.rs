use glam::{Mat4, Vec3, Vec4, usize};
use gltf_json::{
    Accessor, Buffer, Index, Mesh, Node, Scene, Skin, Value,
    accessor::{ComponentType, GenericComponentType},
    buffer::{Stride, Target, View},
    mesh::{Mode, Primitive, Semantic},
    scene::UnitQuaternion,
    validation::{
        Checked::{self, Valid},
        USize64,
    },
};
use log::{info, warn};
use metaverse_messages::capabilities::scene::SceneGroup;
use rgb::bytemuck;
use std::{collections::{BTreeMap, HashMap, HashSet}, f32::consts::FRAC_1_SQRT_2, f64::consts::FRAC_1_SQRT_2};
use std::{borrow::Cow, fs::File, mem, path::PathBuf};

/// This generates a GLTF file and a MeshUpdate for a scenegroup object. This handles the skeleton,
/// mesh deforms, etc.
pub fn generate_avatar_from_scenegroup(
    scene_group: SceneGroup,
    skeleton_path: PathBuf,
    path: PathBuf,
) -> Result<PathBuf, Box<dyn std::error::Error>> {
    let mut root = gltf_json::Root::default();
    let mut raw_buffer = Vec::new();
    let mut nodes = Vec::new();
    let high_path = path.join(format!("{}_high.glb", scene_group.parts[0].name));

    let buffer = root.push(gltf_json::Buffer {
        byte_length: USize64::from(0_usize),
        extensions: Default::default(),
        extras: Default::default(),
        name: Some(scene_group.parts[0].name.clone()),
        uri: None,
    });

    // the root mesh is the first entry in the scene_group mesh parts array. This contains the skin
    // data for the rest of the scene.
    let root_mesh = scene_group.parts[0].clone().sculpt.mesh.unwrap();

    // Load the default skeleton data in from the skeleton path. This is a local file that provides the joint rotations
    // and hierarchy that the scene data does not. The original file can be found in
    // agent/benthic_default_model/skeleton.gltf. It gets copied to the agent share dir.
    info!("Loading skeleton data from {:?}", skeleton_path);
    let (document, buffers, _) = gltf::import(&skeleton_path)
        .unwrap_or_else(|_| panic!("Failed to load skeleton {:?}", skeleton_path));
    let skin = document.skins().next().expect("No skins in gltf");

    // filter out the joint indices thare are used.
    // skin_joint_indices contains the location of the joint in the joint list, for retrieving the
    // joint's inverse bind matrix out of the buffer.
    // joints contains the node index of all of the allowed joints
    let mut skin_joint_indices = Vec::new();
    let mut joints = Vec::new();
    for (i, joint_node) in skin.joints().enumerate() {
        if let Some(name) = joint_node.name() {
            if root_mesh.skin.joint_names.contains(name) {
                skin_joint_indices.push(i);
                joints.push(joint_node.index());
            }
        }
    }

    // This is a little janky, but what it is effectively doing is retrieving the inverse bind
    // matrix of the relevant joint out of the tightly packed buffer data. It then stores this in
    // skin_ibms, which will be used to apply the rotation and scale to the inverse bind matrix
    // data that is retrieved from the server.
    let ibm_accessor = skin
        .inverse_bind_matrices()
        .expect("Skin has no inverse bind matrices");
    let view = ibm_accessor.view().expect("Accessor has no buffer view");
    let buffer_data = &buffers[view.buffer().index()];
    let ibm_offset = ibm_accessor.offset() + view.offset();
    let ibm_stride = view.stride().unwrap_or(16 * 4); // 16 floats * 4 bytes
    let ibm_count = ibm_accessor.count();
    let mut skin_ibms = Vec::new();
    for &joint_i in &skin_joint_indices {
        if joint_i >= ibm_count {
            warn!(
                "Warning: joint index {} out of bounds for IBMs count {}",
                joint_i, ibm_count
            );
            continue;
        }

        let start = ibm_offset + joint_i * ibm_stride;
        let end = start + 16 * 4;
        let matrix_bytes = &buffer_data[start..end];

        let matrix_floats: &[f32] = bytemuck::cast_slice(matrix_bytes);
        let matrix_floats: &[f32; 16] = matrix_floats
            .try_into()
            .expect("Slice with incorrect length");

        let ibm = Mat4::from_cols_array(matrix_floats);
        skin_ibms.push(ibm);
    }

    // This applies the ibm matrix updates from the root mesh's skin.
    // IBMs come in from the server as mostly the identity matrix, except for the w axis, which
    // contains the translation information. In order to obtain the rotations and scale, you need
    // to multiply your local skelet'on's inverse bind matrix, with its w axis as the last part of
    // the identity matrix by the incoming skin's inverse bind matrix.
    let ibm_matrices = root_mesh
        .skin
        .inverse_bind_matrices
        .iter()
        .zip(skin_ibms.iter_mut())
        .map(|(a, b)| {
            let skeleton = Mat4 {
                x_axis: b.x_axis,
                y_axis: b.y_axis,
                z_axis: b.z_axis,
                w_axis: Vec4::new(0.0, 0.0, 0.0, 1.0),
            };
            let corrected_translation = *a;

            skeleton * corrected_translation
        })
        .collect::<Vec<Mat4>>();

    // Start from the pelvis bone and recursively add child joints to the root, to maintain parent
    // child hierarchy.
    let mut joint_index_map = HashMap::new();
    let pelvis_bone_index = skin
        .joints()
        .find(|j| j.name() == Some("mPelvis"))
        .expect("Failed to find pelvis bone")
        .index();

    let skeleton_root = match add_node_recursive(
        &document,
        &mut root,
        &mut joint_index_map,
        pelvis_bone_index,
        &root_mesh.skin.joint_names,
    ) {
        Some(index) => index,
        None => {
            return Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Failed to add skeleton root",
            )));
        }
    };
    nodes.push(Index::new(skeleton_root as u32));
    let mut ibm_bytes_vec = Vec::with_capacity(ibm_matrices.len() * 16 * mem::size_of::<f32>());
    for mat in &ibm_matrices {
        let cols: [f32; 16] = mat.to_cols_array();
        for f in &cols {
            ibm_bytes_vec.extend_from_slice(&f.to_le_bytes());
        }
    }

    let ibm_buffer_view = View {
        buffer,
        byte_length: USize64::from(ibm_bytes_vec.len()),
        byte_offset: Some(USize64::from(raw_buffer.len())),
        byte_stride: None,
        target: None,
        name: Some("inverse_bind_matrices_view".to_string()),
        extensions: Default::default(),
        extras: Default::default(),
    };

    raw_buffer.extend_from_slice(&ibm_bytes_vec);
    let ibm_buffer_view_index = root.push(ibm_buffer_view);
    let ibm_accessor_json = Accessor {
        sparse: None,
        buffer_view: Some(ibm_buffer_view_index),
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

    let inverse_bind_accessor_index = root.push(ibm_accessor_json);
    let new_joints: Vec<Index<gltf_json::Node>> = joints
        .iter()
        .filter_map(|orig_idx| joint_index_map.get(orig_idx))
        .map(|i| Index::new(*i as u32))
        .collect();

    // Create a hashmap to retrieve the index of the parent from the child node.
    // This allows child nodes to look up their parent, instead of parents only knowing about child
    // nodes. This is required in order to calculate the joint-local joint transforms.
    let mut parent_map: HashMap<usize, usize> = HashMap::new();
    for (parent_joint_idx, &parent_joint_node_index) in new_joints.iter().enumerate() {
        let parent_node = &root.nodes[parent_joint_node_index.value()];

        if let Some(children) = &parent_node.children {
            for child_node_index in children {
                if let Some(child_joint_idx) = new_joints
                    .iter()
                    .position(|&j| j.value() == child_node_index.value())
                {
                    parent_map.insert(child_joint_idx, parent_joint_idx);
                }
            }
        }
    }
    // For every joint in the joint node index, retrieve the joint from the root, and if it has a
    // parent, multiply the parent's inverse bind matrix by the child's bind matrix.
    // This calculation allows you to isolate just the transforms between each joint, which will be
    // applied as the local joint translation, rotation and scale.
    for (i, &joint_node_index) in new_joints.iter().enumerate() {
        let node = &mut root.nodes[joint_node_index.value()];
        if let Some(&parent_joint_idx) = parent_map.get(&i) {
            let mat = ibm_matrices[parent_joint_idx] * ibm_matrices[i].inverse();
            let (scale, rotation, translation) = mat.to_scale_rotation_translation();
            node.scale = Some([scale.x, scale.y, scale.z]);
            node.translation = Some([translation.x, translation.y, translation.z]);
            node.rotation = Some(UnitQuaternion([
                rotation.x, rotation.y, rotation.z, rotation.w,
            ]));
        } else {
            let (scale, rotation, translation) = ibm_matrices[i].to_scale_rotation_translation();
            node.scale = Some([scale.x, scale.y, scale.z]);
            node.rotation = Some(UnitQuaternion([
                rotation.x, rotation.y, rotation.z, rotation.w,
            ]));
            node.translation = Some([translation.x, translation.y, translation.z]);
        }
    }
    root.skins.push(Skin {
        joints: new_joints,
        inverse_bind_matrices: Some(inverse_bind_accessor_index),
        skeleton: Some(Index::new(skeleton_root as u32)),
        extensions: Default::default(),
        extras: Default::default(),
        name: skin.name().map(str::to_string),
    });
    // save the skin index to apply to all meshes in the scene.
    // TODO: make sure that this is the right thing to do.
    let skin_index = Index::new((root.skins.len() - 1) as u32);

    for scene in &scene_group.parts {
        if let Some(mesh) = scene.sculpt.mesh.as_ref() {
            let vertices_buffer_length =
                mesh.high_level_of_detail.vertices.len() * mem::size_of::<Vec3>();

            // apply the bind shape matrix to the vertices of the mesh. The bind shape matrix
            // transforms the vertices to give them their proper scale, and align them with the
            // skeleton.
            let vertices_transformed: Vec<Vec3> = mesh
                .high_level_of_detail
                .vertices
                .iter()
                .map(|v| {
                    let v4 = mesh.skin.bind_shape_matrix * Vec4::new(v.x, v.y, v.z, 1.0);
                    Vec3::new(v4.x, v4.y, v4.z)
                })
                .collect();

            let (min, max) = bounding_coords(&vertices_transformed);
            let buffer_view = root.push(View {
                buffer,
                byte_length: USize64::from(vertices_buffer_length),
                byte_offset: Some(USize64::from(raw_buffer.len())),
                byte_stride: Some(Stride(mem::size_of::<Vec3>())),
                extensions: Default::default(),
                extras: Default::default(),
                name: None,
                target: Some(Valid(Target::ArrayBuffer)),
            });
            raw_buffer.extend_from_slice(&to_padded_byte_vector(&vertices_transformed));
            let positions_accessor = root.push(Accessor {
                buffer_view: Some(buffer_view),
                byte_offset: Some(USize64(0)),
                count: USize64::from(mesh.high_level_of_detail.vertices.len()),
                component_type: Valid(GenericComponentType(ComponentType::F32)),
                extensions: Default::default(),
                extras: Default::default(),
                type_: Valid(gltf_json::accessor::Type::Vec3),
                min: Some(Value::from(Vec::from(min))),
                max: Some(Value::from(Vec::from(max))),
                name: None,
                normalized: false,
                sparse: None,
            });

            let index_buffer_length =
                mesh.high_level_of_detail.indices.len() * mem::size_of::<u16>();
            let index_view = root.push(View {
                buffer,
                byte_length: USize64::from(index_buffer_length),
                byte_offset: Some(USize64::from(raw_buffer.len())),
                byte_stride: None,
                target: Some(Valid(Target::ElementArrayBuffer)),
                extensions: None,
                extras: Default::default(),
                name: None,
            });
            for index in &mesh.high_level_of_detail.indices {
                raw_buffer.extend_from_slice(&index.to_le_bytes());
            }
            let index_accessor = root.push(Accessor {
                buffer_view: Some(index_view),
                byte_offset: Some(USize64(0)),
                count: USize64::from(mesh.high_level_of_detail.indices.len()),
                component_type: Valid(GenericComponentType(ComponentType::U16)),
                type_: Valid(gltf_json::accessor::Type::Scalar),
                normalized: false,
                min: None,
                max: None,
                name: None,
                sparse: None,
                extensions: None,
                extras: Default::default(),
            });

            // Add the joint weights which come in with each mesh. The index is the index of the
            // joint the weight acts on within the skin.
            // Weights define how much a movement on the bone affects the vertex. Each vertex can
            // be influcenced by up to four joints. The weights determine how intense that
            // influence is when the joints move.
            let mut joint_indices_bytes = Vec::new();
            let mut joint_weights_bytes = Vec::new();
            for jw in &mesh.high_level_of_detail.weights {
                for &joint in &jw.indices {
                    joint_indices_bytes.extend_from_slice(&joint.to_le_bytes());
                }

                // Convert weights to f32 normalized [0..1]
                let weights_f32: Vec<f32> =
                    jw.weights.iter().map(|&w| (w as f32) / 65535.0).collect();

                let sum: f32 = weights_f32.iter().sum();

                // Normalize weights to sum to 1.0
                // this is crucial or else you will get gltf errors
                let normalized_weights = if sum > 0.0 {
                    weights_f32.iter().map(|w| w / sum).collect::<Vec<f32>>()
                } else {
                    // fallback to equal weights if sum is zero
                    vec![0.25, 0.25, 0.25, 0.25]
                };

                // Write normalized weights to bytes
                for &weight in &normalized_weights {
                    joint_weights_bytes.extend_from_slice(&weight.to_le_bytes());
                }
            }

            let weights_byte_length =
                mesh.high_level_of_detail.weights.len() * 4 * mem::size_of::<u8>();
            let joint_indices_view = root.push(View {
                buffer,
                byte_length: USize64::from(weights_byte_length),
                byte_offset: Some(USize64::from(raw_buffer.len())),
                byte_stride: Some(Stride(mem::size_of::<u8>() * 4)),
                target: Some(Valid(Target::ArrayBuffer)),
                extensions: None,
                extras: Default::default(),
                name: None,
            });
            raw_buffer.extend_from_slice(&joint_indices_bytes);
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

            let indices_byte_length =
                mesh.high_level_of_detail.weights.len() * 4 * mem::size_of::<f32>();
            let joint_weights_view = root.push(View {
                buffer,
                byte_length: USize64::from(indices_byte_length),
                byte_offset: Some(USize64::from(raw_buffer.len())),
                byte_stride: Some(Stride(mem::size_of::<f32>() * 4)), // 4 * u16
                target: Some(Valid(Target::ArrayBuffer)),
                extensions: None,
                extras: Default::default(),
                name: None,
            });
            raw_buffer.extend_from_slice(&joint_weights_bytes);
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
                extensions: Default::default(),
                extras: Default::default(),
                indices: Some(index_accessor),
                material: None,
                mode: Valid(Mode::Triangles),
                targets: None,
            };

            let node_mesh = root.push(Mesh {
                extensions: Default::default(),
                extras: Default::default(),
                name: None,
                primitives: vec![primitive],
                weights: None,
            });
            let node = root.push(Node {
                mesh: Some(node_mesh),
                skin: Some(skin_index),
                ..Default::default()
            });
            nodes.push(node);
        }
    }

    // rotate the model
    // this allows for the model to appear upright in blender, bevy and godot.
    // SL and gltf both use Y-up coordinate systems. This aligns the coordinates to that y-up
    // oirentation, and blender and bevy compensate by applying a 90 degree tilt on load.
    let rotated_root_index = Index::new(root.nodes.len() as u32);
    root.nodes.push(Node {
        camera: None,
        children: Some(nodes.clone()),
        name: Some("RotatedRoot".to_string()),
        rotation: Some(UnitQuaternion([FRAC_1_SQRT_2, FRAC_1_SQRT_2, 0.0, 0.0])),
        ..Default::default()
    });

    root.push(Scene {
        extensions: Default::default(),
        extras: Default::default(),
        name: Some(scene_group.parts[0].name.clone()),
        nodes: vec![rotated_root_index],
    });

    let buffer_length = raw_buffer.len();
    root.buffers[buffer.value()] = Buffer {
        byte_length: USize64::from(buffer_length),
        ..root.buffers[buffer.value()].clone()
    };

    let json_string = gltf_json::serialize::to_string(&root)?;
    let mut json_offset = json_string.len();
    align_to_multiple_of_four(&mut json_offset);

    let glb = gltf::binary::Glb {
        header: gltf::binary::Header {
            magic: *b"glTF",
            version: 2,
            length: (json_offset + buffer_length).try_into()?,
        },
        json: Cow::Owned(json_string.into_bytes()),
        bin: Some(Cow::Owned(raw_buffer)),
    };

    let writer = File::create(&high_path)?;
    glb.to_writer(writer)?;

    Ok(high_path)
}

fn add_node_recursive(
    document: &gltf::Document,
    root: &mut gltf_json::Root,
    joint_index_map: &mut HashMap<usize, usize>,
    node_index: usize,
    allowed_joints: &HashSet<String>,
) -> Option<usize> {
    if let Some(&existing_index) = joint_index_map.get(&node_index) {
        return Some(existing_index);
    }
    let node = document
        .nodes()
        .nth(node_index)
        .expect("Node index out of range");
    if let Some(name) = node.name() {
        if !allowed_joints.contains(name) {
            return None;
        }
    } else {
        return None;
    }
    let children_indices: Vec<usize> = node
        .children()
        .filter_map(|child| {
            add_node_recursive(
                document,
                root,
                joint_index_map,
                child.index(),
                allowed_joints,
            )
        })
        .collect();
    let new_index = root.nodes.len();
    let (translation, rotation, scale) = node.transform().decomposed();
    root.nodes.push(Node {
        camera: None,
        children: if children_indices.is_empty() {
            None
        } else {
            Some(
                children_indices
                    .into_iter()
                    .map(|i| Index::new(i as u32))
                    .collect(),
            )
        },
        skin: None,
        matrix: None,
        mesh: None,
        name: node.name().map(|s| s.to_string()),
        rotation: Some(UnitQuaternion(rotation)),
        scale: Some([scale[0], scale[1], scale[2]]),
        translation: Some([translation[0], translation[1], translation[2]]),
        weights: None,
        extensions: Default::default(),
        extras: Default::default(),
    });
    joint_index_map.insert(node_index, new_index);
    Some(new_index)
}

/// realigns the data to a mutiple of four
fn align_to_multiple_of_four(n: &mut usize) {
    *n = (*n + 3) & !3;
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

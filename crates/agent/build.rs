use glam::Mat4;
use gltf::Node;
use indexmap::IndexMap;
use metaverse_messages::utils::skeleton::Transform;
use metaverse_messages::utils::skeleton::{Joint, JointName, Skeleton};
use std::{collections::HashMap, env, fs::File, io::Write, path::PathBuf, str::FromStr};
use uuid::Uuid;

/// This is used to generate the default keleton from the GLTF file. This allows for creating
/// skeletons with default transforms without having to reread the file every time an avatar loads
/// in.
fn main() {
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let base_skeleton_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("src")
        .join("benthic_default_model")
        .join("skeleton.gltf");

    let skeleton = skeleton_from_gltf(base_skeleton_path);
    let generated_code = generate_skeleton_code(&skeleton);

    let out_file = out_dir.join("default_skeleton.rs");
    let mut file = File::create(out_file).expect("Unable to create skeleton_gen.rs");
    write!(file, "{}", generated_code).expect("unable to write skeleton code");
}

fn skeleton_from_gltf(skeleton_path: PathBuf) -> Skeleton {
    let (document, buffers, _) = gltf::import(&skeleton_path)
        .unwrap_or_else(|_| panic!("Failed to load skeleton {:?}", skeleton_path));
    let skin = document.skins().next().expect("No skins in gltf");

    let nodes: Vec<Node> = document.nodes().collect();
    let ibm_accessor = skin
        .inverse_bind_matrices()
        .expect("Skin has no inverse bind matrices");

    let view = ibm_accessor.view().expect("Accessor has no buffer view");
    let buffer_data = &buffers[view.buffer().index()];
    let ibm_offset = ibm_accessor.offset() + view.offset();
    let ibm_stride = view.stride().unwrap_or(16 * 4); // 16 floats * 4 bytes
    let ibm_count = ibm_accessor.count();

    // Map node index to IBM
    // TODO: This should be moved to build_joint_recursive
    let mut ibm_map: HashMap<usize, Mat4> = HashMap::new();
    for (i, node) in skin.joints().enumerate() {
        if i >= ibm_count {
            panic!(
                "Joint index {} out of bounds for IBMs count {}",
                i, ibm_count
            );
        }

        let start = ibm_offset + i * ibm_stride;
        let end = start + 16 * 4;
        let matrix_bytes = &buffer_data[start..end];
        let matrix_floats: &[f32] = bytemuck::cast_slice(matrix_bytes);
        let matrix_floats: &[f32; 16] = matrix_floats
            .try_into()
            .expect("Invalid matrix slice length");
        ibm_map.insert(node.index(), Mat4::from_cols_array(matrix_floats));
    }

    let mut joints = IndexMap::new();
    // 158 is the index of mpelvis
    build_joint_recursive(158, None, 0, &nodes, &mut joints, &ibm_map);
    Skeleton {
        joints,
        root: vec![JointName::Pelvis],
    }
}

fn build_joint_recursive(
    index: usize,
    parent: Option<JointName>,
    parent_index: usize,
    nodes: &[Node],
    joints: &mut IndexMap<JointName, Joint>,
    ibm_map: &HashMap<usize, Mat4>,
) {
    let node = nodes[index].clone();
    let name = JointName::from_str(node.name().unwrap()).unwrap();
    if joints.contains_key(&name) {
        return;
    }

    let mut children = Vec::new();
    for child in node.children() {
        children.push(
            JointName::from_str(child.name().unwrap())
                .unwrap_or_else(|err| panic!("errored on {:?}, {:?}", child.name(), err)),
        );
        build_joint_recursive(
            child.index(),
            Some(name),
            node.index(),
            nodes,
            joints,
            ibm_map,
        );
    }
    let global = ibm_map[&index];
    let local = if index == parent_index {
        global
    } else {
        ibm_map[&parent_index] * global.inverse()
    };
    let joint = Joint {
        name,
        parent,
        children,
        transforms: vec![Transform {
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
    };
    joints.insert(name, joint);
}

fn generate_skeleton_code(skeleton: &Skeleton) -> String {
    let joints_code = skeleton
        .joints
        .iter()
        .map(|(name, joint)| {
            let children = joint
                .children
                .iter()
                .map(|c| format!("JointName::{:?}", c))
                .collect::<Vec<_>>()
                .join(", ");

            let transform = joint.transforms[0].transform.to_cols_array();
            let transform_str = format!(
                "Mat4::from_cols_array(&[{}])",
                transform
                    .iter()
                    .map(|f| format!("{:?}", f))
                    .collect::<Vec<_>>()
                    .join(", ")
            );

            let local_transform = joint.local_transforms[0].transform.to_cols_array();
            let local_transform_str = format!(
                "Mat4::from_cols_array(&[{}])",
                local_transform
                    .iter()
                    .map(|f| format!("{:?}", f))
                    .collect::<Vec<_>>()
                    .join(", ")
            );

            format!(
                "(JointName::{n}, Joint {{
                name: JointName::{n},
                parent: {parent},
                children: vec![{children}],
                transforms: vec![
                    Transform{{
                        name:\"Default\".to_string(), 
                        id: Uuid::parse_str(\"{uuid}\").unwrap(), 
                        transform:{transform},
                        rank: 0
                    }}],
                local_transforms: vec![
                    Transform{{
                        name:\"Default\".to_string(), 
                        id: Uuid::parse_str(\"{uuid}\").unwrap(), 
                        transform:{local_transform},
                        rank: 0
                    }}],
                }})",
                n = format!("{:?}", name),
                parent = match &joint.parent {
                    Some(p) => format!("Some(JointName::{:?})", p),
                    None => "None".to_string(),
                },
                children = children,
                uuid = Uuid::nil(),
                transform = transform_str,
                local_transform = local_transform_str,
            )
        })
        .collect::<Vec<_>>()
        .join(",\n");

    format!(
        "Skeleton {{
            joints: vec![{}].into_iter().collect(),
            root: vec![JointName::Pelvis],
        }}",
        joints_code
    )
}

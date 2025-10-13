use glam::Vec3;
use gltf_json::validation::Checked::Valid;
use gltf_json::validation::USize64;
use metaverse_messages::http::mesh::Mesh;
use std::{borrow::Cow, mem};
use std::{fs::File, path::PathBuf};

/// Generates gltf file out of triangles retrieved from the server
pub fn generate_land_gltf(
    mesh: &Mesh,
    path: PathBuf,
    name: String,
) -> Result<PathBuf, Box<dyn std::error::Error>> {
    let path = path.join(format!("{}_high.gltf", name));

    let mut root = gltf_json::Root::default();
    let buffer = root.push(gltf_json::Buffer {
        byte_length: USize64::from(0_usize),
        extensions: Default::default(),
        extras: Default::default(),
        name: Some(name),
        uri: None,
    });

    let meshdata = mesh.high_level_of_detail.triangles.as_ref().unwrap();
    let buffer_length = meshdata.len() * mem::size_of::<Vec3>();
    let (min, max) = bounding_coords(meshdata);
    let buffer_view = root.push(gltf_json::buffer::View {
        buffer,
        byte_length: USize64::from(buffer_length),
        byte_offset: Some(USize64(0)),
        byte_stride: Some(gltf_json::buffer::Stride(mem::size_of::<Vec3>())),
        extensions: Default::default(),
        extras: Default::default(),
        name: None,
        target: Some(Valid(gltf_json::buffer::Target::ArrayBuffer)),
    });
    let positions = root.push(gltf_json::Accessor {
        buffer_view: Some(buffer_view),
        byte_offset: Some(USize64(0)),
        count: USize64::from(meshdata.len()),
        component_type: Valid(gltf_json::accessor::GenericComponentType(
            gltf_json::accessor::ComponentType::F32,
        )),
        extensions: Default::default(),
        extras: Default::default(),
        type_: Valid(gltf_json::accessor::Type::Vec3),
        min: Some(gltf_json::Value::from(Vec::from(min))),
        max: Some(gltf_json::Value::from(Vec::from(max))),
        name: None,
        normalized: false,
        sparse: None,
    });

    let primitive = gltf_json::mesh::Primitive {
        attributes: {
            let mut map = std::collections::BTreeMap::new();
            map.insert(Valid(gltf_json::mesh::Semantic::Positions), positions);
            map
        },
        extensions: Default::default(),
        extras: Default::default(),
        indices: None,
        material: None,
        mode: Valid(gltf_json::mesh::Mode::Triangles),
        targets: None,
    };
    let mesh = root.push(gltf_json::Mesh {
        extensions: Default::default(),
        extras: Default::default(),
        name: None,
        primitives: vec![primitive],
        weights: None,
    });
    let node = root.push(gltf_json::Node {
        mesh: Some(mesh),
        ..Default::default()
    });

    root.push(gltf_json::Scene {
        extensions: Default::default(),
        extras: Default::default(),
        name: None,
        nodes: vec![node],
    });

    let json_string = gltf_json::serialize::to_string(&root)?;
    let mut json_offset = json_string.len();
    align_to_multiple_of_four(&mut json_offset);
    let glb = gltf::binary::Glb {
        header: gltf::binary::Header {
            magic: *b"glTF",
            version: 2,
            length: (json_offset + buffer_length).try_into()?,
        },
        bin: Some(Cow::Owned(to_padded_byte_vector(meshdata))),
        json: Cow::Owned(json_string.into_bytes()),
    };

    let writer = File::create(&path)?;
    glb.to_writer(writer)?;

    Ok(path)
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

/// realigns the data to a mutiple of four
fn align_to_multiple_of_four(n: &mut usize) {
    *n = (*n + 3) & !3;
}

/// Converts a byte vector to a vector aligned to a mutiple of 4
fn to_padded_byte_vector(data: &[Vec3]) -> Vec<u8> {
    let flat: Vec<[f32; 3]> = data.iter().map(|v| [v.x, v.y, v.z]).collect();
    let byte_slice: &[u8] = bytemuck::cast_slice(&flat);
    let mut new_vec: Vec<u8> = byte_slice.to_owned();

    while !new_vec.len().is_multiple_of(4) {
        new_vec.push(0); // pad to multiple of four bytes
    }

    new_vec
}

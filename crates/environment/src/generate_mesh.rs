use crate::generate_terrain::Land;
use std::{borrow::Cow, fs::{create_dir_all, File}, io, mem};
use gltf_json::validation::{USize64, Checked::Valid};
use log::info;

#[derive(Copy, Clone, Debug, bytemuck::NoUninit)]
#[repr(C)]
struct Vertex{
    position: [f32; 3]
}

pub fn generate_land_mesh(patch: &Land) -> Result<(), Box<dyn std::error::Error>> {
    let scale = 1.0;
    let mut triangles: Vec<Vertex> = Vec::new();

    let grid_size = patch.terrain_header.patch_size;
    for row in 0..grid_size - 1 {
        for col in 0..grid_size - 1 {
            let top_left = row * grid_size + col;
            let top_right = top_left + 1;
            let bottom_left = (row + 1) * grid_size + col;
            let bottom_right = bottom_left + 1;

            let x0 = col as f32 * scale;
            let y0 = patch.heightmap[top_left as usize] * scale;
            let z0 = row as f32 * scale;

            let x1 = (col + 1) as f32 * scale;
            let y1 = patch.heightmap[top_right as usize] * scale;
            let z1 = row as f32 * scale;

            let x2 = col as f32 * scale;
            let y2 = patch.heightmap[bottom_left as usize] * scale;
            let z2 = (row + 1) as f32 * scale;

            let x3 = (col + 1) as f32 * scale;
            let y3 = patch.heightmap[bottom_right as usize] * scale;
            let z3 = (row + 1) as f32 * scale;

            triangles.push(Vertex{position:[x0, y0, z0]});
            triangles.push(Vertex{position:[x2, y2, z2]});
            triangles.push(Vertex{position:[x1, y1, z1]}); 
            triangles.push(Vertex{position: [x1, y1, z1]});
            triangles.push(Vertex{position: [x2, y2, z2]});
            triangles.push(Vertex{position: [x3, y3, z3]});

        }
    }

    let mut root = gltf_json::Root::default();
    let buffer_length = triangles.len() * mem::size_of::<f32>() * 3;

    let buffer = root.push(gltf_json::Buffer{
        byte_length: USize64::from(buffer_length), 
        extensions: Default::default(),
        extras: Default::default(),
        name: None, 
        uri: None
    });
    let buffer_view = root.push(gltf_json::buffer::View{
        buffer,
        byte_length: USize64::from(buffer_length),
        byte_offset: None,
        byte_stride: Some(gltf_json::buffer::Stride(mem::size_of::<Vertex>())),
        extensions: Default::default(),
        extras: Default::default(),
        name:None, 
        target:Some(Valid(gltf_json::buffer::Target::ArrayBuffer))
    });
    let positions = root.push(gltf_json::Accessor{
        buffer_view: Some(buffer_view),
        byte_offset: Some(USize64(0)),
        count: USize64::from(triangles.len()),
        component_type: Valid(gltf_json::accessor::GenericComponentType(gltf_json::accessor::ComponentType::F32)),
        extensions: Default::default(),
        extras: Default::default(),
        type_: Valid(gltf_json::accessor::Type::Vec3),
        min:None, 
        max:None,
        name: None, 
        normalized: false, 
        sparse:None
    });
    let primitive = gltf_json::mesh::Primitive{
        attributes:{
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
    let mesh = root.push(gltf_json::Mesh{
        extensions: Default::default(),
        extras: Default::default(),
        name: None, 
        primitives: vec![primitive],
        weights: None
    });
    let node = root.push(gltf_json::Node{
        mesh: Some(mesh),
        ..Default::default()
    });
    root.push(gltf_json::Scene{
        extensions: Default::default(),
        extras: Default::default(),
        name: None, 
        nodes: vec![node]
    });
    let json_string = gltf_json::serialize::to_string(&root)?;
    let mut json_offset = json_string.len(); 
    align_to_multiple_of_four(&mut json_offset);
    let glb = gltf::binary::Glb{
        header: gltf::binary::Header{
            magic: *b"glTF",
            version: 2, 
            length: (json_offset + buffer_length).try_into()?
        },
        bin: Some(Cow::Owned(to_padded_byte_vector(&triangles))),
        json: Cow::Owned(json_string.into_bytes()),
    };

    if let Some(data_dir) = dirs::data_dir(){
        let local_share_dir = data_dir.join("benthic");
        if !local_share_dir.exists(){
            create_dir_all(&local_share_dir)?;
            info!("Created Directory: {:?}", local_share_dir);
        }
        let land_dir = local_share_dir.join("land");
        if !land_dir.exists(){
            create_dir_all(&land_dir)?;
            info!("Created Directory: {:?}", land_dir);
        }
        let file_path = land_dir.join(format!("{}.glb", patch.terrain_header.filename));
        let writer = File::create(file_path)?;
        glb.to_writer(writer)?;
        Ok(())
    } else {
        Err(Box::new(io::Error::new(io::ErrorKind::NotFound, "Failed to find data directory")))
    }
}

fn align_to_multiple_of_four(n: &mut usize) {
    *n = (*n + 3) & !3;
}
fn to_padded_byte_vector<T: bytemuck::NoUninit>(data: &[T]) -> Vec<u8> {
    let byte_slice: &[u8] = bytemuck::cast_slice(data);
    let mut new_vec: Vec<u8> = byte_slice.to_owned();

    while new_vec.len() % 4 != 0 {
        new_vec.push(0); // pad to multiple of four bytes
    }

    new_vec
}


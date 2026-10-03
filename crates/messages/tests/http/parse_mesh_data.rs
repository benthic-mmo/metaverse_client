use std::{fs::File, io::Read};

use metaverse_messages::http::mesh::Mesh;
use serde_llsd_benthic::binary_from_bytes;

#[test]
fn handle_mesh_data() {
    let mut file = File::open("tests/data/mesh_data.txt").unwrap();
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer).unwrap();

    let mesh = Mesh::from_bytes(&buffer).unwrap();
    assert!(mesh.skin.is_some());
}

#[test]
fn handle_mesh_with_collision() {
    let mut file = File::open("tests/data/mesh_with_collision.bin").unwrap();
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer).unwrap();

    let mesh = Mesh::from_bytes(&buffer).unwrap();
    assert!(mesh.skin.is_some());
}

#[test]
fn handle_mesh_2() {
    let bytes = std::fs::read("tests/data/mesh_2.bin").unwrap();

    for (i, chunk) in bytes.chunks(16).enumerate() {
        println!(
            "{:08x}: {}",
            i * 16,
            chunk
                .iter()
                .map(|b| format!("{b:02x}"))
                .collect::<Vec<_>>()
                .join(" ")
        );
    }

    let result = binary_from_bytes(&bytes).unwrap();
}

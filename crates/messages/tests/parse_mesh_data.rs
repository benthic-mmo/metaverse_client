use std::{fs::File, io::Read};

use metaverse_messages::capabilities::mesh_data::Mesh;

#[test]
fn handle_mesh_data(){
    let mut file = File::open("tests/mesh_data.txt").unwrap();
    let mut buffer = Vec::new();

    file.read_to_end(&mut buffer).unwrap();

    Mesh::from_bytes(&buffer).unwrap();
}

use std::{fs::File, io::Read};

use metaverse_messages::http::scene::SceneGroup;

#[test]
fn parse_scenegroup_1() {
    let mut file1 = File::open("tests/data/scenegroup_1.txt").unwrap();
    let mut buffer1 = Vec::new();

    file1.read_to_end(&mut buffer1).unwrap();

    let scenegroup1 = SceneGroup::from_xml(&buffer1).unwrap();
    println!("{:?}", scenegroup1.parts[0].shape.texture);
}

#[test]
fn parse_scenegroup_2() {
    let mut file2 = File::open("tests/data/scenegroup_2.txt").unwrap();
    let mut buffer2 = Vec::new();

    file2.read_to_end(&mut buffer2).unwrap();

    let scenegroup2 = SceneGroup::from_xml(&buffer2).unwrap();

    println!("{:?}", scenegroup2.parts[0].shape.texture);
}

#[test]
fn parse_scenegroup_3() {
    let mut file3 = File::open("tests/data/scenegroup_3.txt").unwrap();
    let mut buffer3 = Vec::new();

    file3.read_to_end(&mut buffer3).unwrap();

    let scenegroup3 = SceneGroup::from_xml(&buffer3).unwrap();

    println!("{:?}", scenegroup3.parts[0].shape.texture);
}

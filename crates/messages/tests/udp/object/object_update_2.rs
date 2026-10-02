use std::{fs::File, io::Read};

use metaverse_messages::packet::packet_protocol::Packet;

#[test]
fn decode_object_2() {
    let mut file = File::open("tests/data/objectupdate_2.txt").unwrap();
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer).unwrap();
    let packet = match Packet::from_bytes(&buffer) {
        Ok(p) => p,
        Err(e) => {
            panic!("Failed to create packet: {}, ", e)
        }
    };
}

#[test]
fn decode_object_3() {
    let mut file = File::open("tests/data/unknown.bin").unwrap();
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer).unwrap();
    let packet = match Packet::from_bytes(&buffer) {
        Ok(p) => p,
        Err(e) => {
            panic!("Failed to create packet: {}", e)
        }
    };
}

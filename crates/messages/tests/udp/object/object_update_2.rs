use std::{fs::File, io::Read};

use metaverse_messages::packet::packet_protocol::Packet;

#[test]
fn decode_unknown_packet() {
    let mut file = File::open("tests/data/BODY_DECODE_ERROR.txt").unwrap();
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer).unwrap();
    let packet = match Packet::from_bytes(&buffer) {
        Ok(p) => p,
        Err(e) => {
            panic!("Failed to create packet: {}, ", e)
        }
    };
}

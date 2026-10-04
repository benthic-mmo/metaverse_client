use std::{any::Any, fs::File, io::Read};

use metaverse_messages::{
    packet::packet_protocol::Packet, udp::core::generic_streaming_message::GenericStreamingMessage,
};

#[test]
fn generic_streaming_message_1() {
    let mut file = File::open("tests/data/generic_streaming_message.bin").unwrap();
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer).unwrap();
    let packet = match Packet::from_bytes(&buffer) {
        Ok(p) => p,
        Err(e) => {
            panic!("Failed to create packet: {}", e)
        }
    };
    println!("{:?}", packet);
}

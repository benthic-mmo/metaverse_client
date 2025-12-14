use crate::errors::ParseError;
use crate::packet::{
    header::{Header, PacketFrequency},
    packet::{Packet, PacketData},
    packet_types::PacketType,
};
use std::io::Cursor;

impl Packet {
    /// create a new test packet
    pub fn new_test_packet(test_packet: TestPacket) -> Self {
        Packet {
            header: Header {
                id: 1,
                reliable: true,
                zerocoded: false,
                frequency: PacketFrequency::Low,
                ..Default::default()
            },
            body: PacketType::TestPacket(Box::new(test_packet)),
        }
    }
}

#[derive(Debug, Clone)]
/// TODO: UNIMPLEMENTED
pub struct TestPacket {}

impl PacketData for TestPacket {
    fn from_bytes(bytes: &[u8]) -> Result<Self, ParseError> {
        let mut _cursor = Cursor::new(bytes);
        Ok(TestPacket {})
    }
    fn to_bytes(&self) -> Vec<u8> {
        Vec::new()
    }
}

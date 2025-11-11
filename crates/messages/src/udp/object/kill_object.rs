use crate::errors::ParseError;
use crate::packet::{
    header::{Header, PacketFrequency},
    packet::{Packet, PacketData},
    packet_types::PacketType,
};
use std::io::Cursor;

impl Packet {
    /// create a new kill object packet
    pub fn new_kill_object(kill_object: KillObject) -> Self {
        Packet {
            header: Header {
                id: 16,
                reliable: true,
                resent: false,
                zerocoded: false,
                appended_acks: false,
                sequence_number: 0,
                frequency: PacketFrequency::High,
                ack_list: None,
                size: None,
            },
            body: PacketType::KillObject(Box::new(kill_object)),
        }
    }
}

#[derive(Debug, Clone)]
/// TODO: UNIMPLEMENTED
pub struct KillObject {}

impl PacketData for KillObject {
    fn from_bytes(bytes: &[u8]) -> Result<Self, ParseError> {
        let mut _cursor = Cursor::new(bytes);
        Ok(KillObject {})
    }
    fn to_bytes(&self) -> Vec<u8> {
        
        Vec::new()
    }
}

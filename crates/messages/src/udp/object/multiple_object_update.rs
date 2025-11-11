use crate::errors::ParseError;
use crate::packet::{
    header::{Header, PacketFrequency},
    packet::{Packet, PacketData},
    packet_types::PacketType,
};
use std::io::Cursor;

impl Packet {
    /// create a new multiple object update packet
    pub fn new_multiple_object_update(multiple_object_update: MultipleObjectUpdate) -> Self {
        Packet {
            header: Header {
                id: 2,
                reliable: true,
                resent: false,
                zerocoded: false,
                appended_acks: false,
                sequence_number: 0,
                frequency: PacketFrequency::Medium,
                ack_list: None,
                size: None,
            },
            body: PacketType::MultipleObjectUpdate(Box::new(multiple_object_update)),
        }
    }
}

#[derive(Debug, Clone)]
/// TODO: UNIMPLLEMENTED
pub struct MultipleObjectUpdate {}

impl PacketData for MultipleObjectUpdate {
    fn from_bytes(bytes: &[u8]) -> Result<Self, ParseError> {
        let mut _cursor = Cursor::new(bytes);
        Ok(MultipleObjectUpdate {})
    }
    fn to_bytes(&self) -> Vec<u8> {
        
        Vec::new()
    }
}

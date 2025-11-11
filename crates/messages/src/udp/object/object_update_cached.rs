use crate::errors::ParseError;
use crate::packet::{
    header::{Header, PacketFrequency},
    packet::{Packet, PacketData},
    packet_types::PacketType,
};
use std::io::Cursor;

impl Packet {
    /// create a new object update cached packet
    pub fn new_object_update_cached(object_update_cached: ObjectUpdateCached) -> Self {
        Packet {
            header: Header {
                id: 14,
                reliable: true,
                resent: false,
                zerocoded: false,
                appended_acks: false,
                sequence_number: 0,
                frequency: PacketFrequency::High,
                ack_list: None,
                size: None,
            },
            body: PacketType::ObjectUpdateCached(Box::new(object_update_cached)),
        }
    }
}

#[derive(Debug, Clone)]
/// TODO: UNIMPLEMENTED
pub struct ObjectUpdateCached {}

impl PacketData for ObjectUpdateCached {
    fn from_bytes(bytes: &[u8]) -> Result<Self, ParseError> {
        let mut _cursor = Cursor::new(bytes);
        Ok(ObjectUpdateCached {})
    }
    fn to_bytes(&self) -> Vec<u8> {
        
        Vec::new()
    }
}

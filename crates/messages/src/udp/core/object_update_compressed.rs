use crate::errors::ParseError;
use crate::packet::{
    header::{Header, PacketFrequency},
    packet::{Packet, PacketData},
    packet_types::PacketType,
};

use std::io::{self, Cursor};

impl Packet {
    pub fn new_object_update_compressed(object_update_compressed: ObjectUpdateCompressed) -> Self {
        Packet {
            header: Header {
                id: 13,
                reliable: true,
                resent: false,
                zerocoded: false,
                appended_acks: false,
                sequence_number: 0,
                frequency: PacketFrequency::High,
                ack_list: None,
                size: None,
            },
            body: PacketType::ObjectUpdateCompressed(Box::new(object_update_compressed)),
        }
    }
}

/// add your struct fields here
#[derive(Debug, Clone)]
pub struct ObjectUpdateCompressed {}

impl PacketData for ObjectUpdateCompressed {
    fn from_bytes(bytes: &[u8]) -> Result<Self, ParseError> {
        let mut cursor = Cursor::new(bytes);
        // handle from bytes
        Ok(ObjectUpdateCompressed{
            // Struct fields 
        })
    }
    fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        // push your data into the new vector
        bytes
    }
}

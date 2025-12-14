use crate::errors::ParseError;
use crate::packet::{
    header::{Header, PacketFrequency},
    packet::{Packet, PacketData},
    packet_types::PacketType,
};

use std::io::Cursor;

impl Packet {
    /// create a new object update compressed packet
    pub fn new_object_update_compressed(object_update_compressed: ObjectUpdateCompressed) -> Self {
        Packet {
            header: Header {
                id: 13,
                reliable: true,
                zerocoded: false,
                frequency: PacketFrequency::High,
                ..Default::default()
            },
            body: PacketType::ObjectUpdateCompressed(Box::new(object_update_compressed)),
        }
    }
}

#[derive(Debug, Clone)]
/// TODO: UNIMPLEMENTED
pub struct ObjectUpdateCompressed {}

impl PacketData for ObjectUpdateCompressed {
    fn from_bytes(bytes: &[u8]) -> Result<Self, ParseError> {
        let mut _cursor = Cursor::new(bytes);
        Ok(ObjectUpdateCompressed {})
    }
    fn to_bytes(&self) -> Vec<u8> {
        // push your data into the new vector
        Vec::new()
    }
}

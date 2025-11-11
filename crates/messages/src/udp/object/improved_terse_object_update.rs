use crate::errors::ParseError;
use crate::packet::{
    header::{Header, PacketFrequency},
    packet::{Packet, PacketData},
    packet_types::PacketType,
};
use std::io::Cursor;

impl Packet {
    /// create a new improved terse object update packet
    pub fn new_improved_terse_object_update(
        improved_terse_object_update: ImprovedTerseObjectUpdate,
    ) -> Self {
        Packet {
            header: Header {
                id: 15,
                reliable: true,
                resent: false,
                zerocoded: false,
                appended_acks: false,
                sequence_number: 0,
                frequency: PacketFrequency::High,
                ack_list: None,
                size: None,
            },
            body: PacketType::ImprovedTerseObjectUpdate(Box::new(improved_terse_object_update)),
        }
    }
}

#[derive(Debug, Clone)]
/// TODO: UNIMPLEMENTED
pub struct ImprovedTerseObjectUpdate {}

impl PacketData for ImprovedTerseObjectUpdate {
    fn from_bytes(bytes: &[u8]) -> Result<Self, ParseError> {
        let mut _cursor = Cursor::new(bytes);
        Ok(ImprovedTerseObjectUpdate {})
    }
    fn to_bytes(&self) -> Vec<u8> {
        
        Vec::new()
    }
}

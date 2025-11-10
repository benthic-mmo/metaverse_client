use crate::errors::ParseError;
use crate::packet::{
    header::{Header, PacketFrequency},
    packet::{Packet, PacketData},
    packet_types::PacketType,
};
/// this is a file for easily creating a new packet.
/// Simply copy this and fill in the data to create a new packet
/// *local_name*    is something like "region_handshake"
/// *PacketName*    is the name of the packet like "RegionHandshake"
/// *id*            is the ID of the packet
///
use std::io::Cursor;

impl Packet {
    pub fn new_viewer_effect(viewer_effect: ViewerEffect) -> Self {
        Packet {
            header: Header {
                id: 17,
                reliable: false,
                resent: false,
                zerocoded: false,
                appended_acks: false,
                sequence_number: 0,
                frequency: PacketFrequency::Medium,
                ack_list: None,
                size: None,
            },
            body: PacketType::ViewerEffect(Box::new(viewer_effect)),
        }
    }
}

/// add your struct fields here
#[derive(Debug, Clone)]
pub struct ViewerEffect {}

impl PacketData for ViewerEffect {
    fn from_bytes(bytes: &[u8]) -> Result<Self, ParseError> {
        let mut cursor = Cursor::new(bytes);
        // handle from bytes
        Ok(ViewerEffect{
            // Struct fields 
        })
    }
    fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        // push your data into the new vector
        bytes
    }
}

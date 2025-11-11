use crate::errors::ParseError;
use crate::packet::{
    header::{Header, PacketFrequency},
    packet::{Packet, PacketData},
    packet_types::PacketType,
};
use std::io::Cursor;

impl Packet {
    /// create a new viewer effect packet
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

#[derive(Debug, Clone)]
/// TODO: UNIMPLEMENTED
pub struct ViewerEffect {}

impl PacketData for ViewerEffect {
    fn from_bytes(bytes: &[u8]) -> Result<Self, ParseError> {
        let mut _cursor = Cursor::new(bytes);
        Ok(ViewerEffect {})
    }
    fn to_bytes(&self) -> Vec<u8> {
        
        Vec::new()
    }
}

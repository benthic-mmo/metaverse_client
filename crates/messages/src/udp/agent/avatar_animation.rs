use crate::errors::ParseError;
use crate::packet::{
    header::{Header, PacketFrequency},
    packet::{Packet, PacketData},
    packet_types::PacketType,
};
use std::io::Cursor;

impl Packet {
    /// create a new avatar animation packet
    pub fn new_avatar_animation(avatar_animation: AvatarAnimation) -> Self {
        Packet {
            header: Header {
                id: 20,
                reliable: true,
                zerocoded: false,
                frequency: PacketFrequency::High,
                ..Default::default()
            },
            body: PacketType::AvatarAnimation(Box::new(avatar_animation)),
        }
    }
}

#[derive(Debug, Clone)]
/// TODO: UNIMPLEMENTED
pub struct AvatarAnimation {}

impl PacketData for AvatarAnimation {
    fn from_bytes(bytes: &[u8]) -> Result<Self, ParseError> {
        let mut _cursor = Cursor::new(bytes);
        Ok(AvatarAnimation {})
    }
    fn to_bytes(&self) -> Vec<u8> {
        Vec::new()
    }
}

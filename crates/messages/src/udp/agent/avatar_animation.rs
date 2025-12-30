use byteorder::{LittleEndian, ReadBytesExt};
use uuid::Uuid;

use crate::errors::ParseError;
use crate::packet::{
    header::{Header, PacketFrequency},
    packet::{Packet, PacketData},
    packet_types::PacketType,
};
use std::io::{Cursor, Read};

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
pub struct AvatarAnimation {
    pub sender_id: Uuid,
    pub animations: Vec<AnimationEntry>,
    pub sources: Vec<Uuid>,
}

#[derive(Debug, Clone)]
pub struct AnimationEntry {
    pub anim_id: Uuid,
    pub sequence_id: i32,
}

impl PacketData for AvatarAnimation {
    fn from_bytes(bytes: &[u8]) -> Result<Self, ParseError> {
        let mut cursor = Cursor::new(bytes);
        let mut buf = [0u8; 16];
        cursor.read_exact(&mut buf)?;
        let sender_id = Uuid::from_bytes(buf);

        let anim_count = cursor.read_u8()? as usize;
        let mut animations = Vec::with_capacity(anim_count);

        for _ in 0..anim_count {
            let mut buf = [0u8; 16];
            cursor.read_exact(&mut buf)?;
            let anim_id = Uuid::from_bytes(buf);
            let sequence_id = cursor.read_i32::<LittleEndian>()?;
            animations.push(AnimationEntry {
                anim_id,
                sequence_id,
            });
        }

        let source_count = cursor.read_u8()? as usize;
        let mut sources = Vec::with_capacity(source_count);

        for _ in 0..source_count {
            let mut buf = [0u8; 16];
            cursor.read_exact(&mut buf)?;
            sources.push(Uuid::from_bytes(buf));
        }

        let anim = AvatarAnimation {
            sender_id,
            animations,
            sources,
        };
        Ok(anim)
    }
    fn to_bytes(&self) -> Vec<u8> {
        Vec::new()
    }
}

use byteorder::{LittleEndian, ReadBytesExt};
use std::io::{Cursor, Read};
use uuid::Uuid;

use crate::{
    errors::ParseError,
    packet::{
        header::{Header, PacketFrequency},
        packet::{Packet, PacketData},
        packet_types::PacketType,
    },
};

impl Packet {
    /// creates a new avatar appearance packet
    pub fn new_avatar_appearance(avatar_appearance: AvatarAppearance) -> Self {
        Packet {
            header: Header {
                id: 158,
                reliable: true,
                resent: false,
                zerocoded: false,
                appended_acks: false,
                sequence_number: 0,
                frequency: PacketFrequency::Low,
                ack_list: None,
                size: None,
            },
            body: PacketType::AvatarAppearance(Box::new(avatar_appearance)),
        }
    }
}

#[derive(Debug, Clone, Default)]
/// Avatar Appearance struct
pub struct AvatarAppearance {
    /// the ID of the user
    pub id: Uuid,
    /// is the user a trial user
    pub is_trial: bool,
    /// the bytes containing the texture data
    pub texture_data: Vec<u8>,
    /// the byes containing the visual param data
    pub visual_param_data: Vec<u8>,
    /// the bytes containing the remaining data
    pub remaining_data: Vec<u8>,
}

impl PacketData for AvatarAppearance {
    fn from_bytes(bytes: &[u8]) -> Result<Self, ParseError> {
        let mut cursor = Cursor::new(bytes);

        let mut id_bytes = [0u8; 16];
        cursor.read_exact(&mut id_bytes)?;
        let id = Uuid::from_bytes(id_bytes);

        let is_trial = cursor.read_u8()? != 0;
        let length = cursor.read_u16::<LittleEndian>()?;

        let end = cursor.position() + length as u64;
        let texture_data = cursor.get_ref()[cursor.position() as usize..end as usize].to_vec();
        cursor.set_position(end);

        let length = cursor.read_u8()?;
        let end = cursor.position() + length as u64;
        let visual_param_data = cursor.get_ref()[cursor.position() as usize..end as usize].to_vec();
        cursor.set_position(end);

        let mut remaining_data = Vec::new();
        cursor.read_to_end(&mut remaining_data)?;

        Ok(AvatarAppearance {
            id,
            is_trial,
            texture_data,
            visual_param_data,
            remaining_data,
        })
    }

    fn to_bytes(&self) -> Vec<u8> {
        vec![]
    }
}

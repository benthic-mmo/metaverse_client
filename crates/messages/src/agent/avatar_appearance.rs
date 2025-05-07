use glam::Vec3;
use std::io::{Cursor, Read};
use uuid::Uuid;

use crate::packet::{
    header::{Header, PacketFrequency},
    packet::{Packet, PacketData},
    packet_types::PacketType,
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

#[derive(Debug, Clone)]
/// Avatar Appearance struct
pub struct AvatarAppearance {
    id: Uuid,
    is_trial: bool,
    texture_entry: String,
    visual_params: VisualParam,
    appearance_data: Vec<AppearanceData>,
    hover_height: Vec3,
}

#[derive(Debug, Clone)]
/// slider value of visual parameters
pub struct VisualParam {}

#[derive(Debug, Clone)]
pub struct AppearanceData {
    appearance_version: u8,
    current_outfit_folder_version: f32,
    flags: u32,
}

impl PacketData for AvatarAppearance {
    fn from_bytes(bytes: &[u8]) -> std::io::Result<Self> {
        let mut cursor = Cursor::new(bytes);
        let mut id_bytes = [0u8; 16];
        cursor.read_exact(&mut id_bytes)?;
        let id = Uuid::from_bytes(id_bytes);

        Ok(AvatarAppearance {
            id,
            is_trial: false,
            texture_entry: "".to_string(),
            visual_params: VisualParam {},
            appearance_data: vec![AppearanceData {
                appearance_version: 0,
                current_outfit_folder_version: 0.0,
                flags: 0,
            }],
            hover_height: Vec3 {
                x: 0.0,
                y: 0.0,
                z: 0.0,
            },
        })
    }

    fn to_bytes(&self) -> Vec<u8> {
        vec![]
    }
}

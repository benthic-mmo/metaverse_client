use crate::errors::ParseError;
use crate::packet::{
    header::{Header, PacketFrequency},
    packet::{Packet, PacketData},
    packet_types::PacketType,
};
use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use glam::Vec3;
use std::io::{Cursor, Read};
use uuid::Uuid;

impl Packet {
    /// create a new teleport request packet
    pub fn new_teleport_request(teleport_request: TeleportRequest) -> Self {
        Packet {
            header: Header {
                id: 62,
                reliable: true,
                zerocoded: false,
                frequency: PacketFrequency::Low,
                ..Default::default()
            },
            body: PacketType::TeleportRequest(Box::new(teleport_request)),
        }
    }
}

/// Packet describing a new teleport request.
#[derive(Debug, Clone)]
pub struct TeleportRequest {
    /// The agent ID sending the teleport request
    pub agent_id: Uuid,
    /// the session ID sending the teleport request
    pub session_id: Uuid,
    /// the ID of the destination region
    pub region_id: Uuid,
    /// the position in the region where the teleport is going to
    pub position: Vec3,
    /// the direction the player will face after teleport
    pub look_at: Vec3,
}

impl PacketData for TeleportRequest {
    fn from_bytes(bytes: &[u8]) -> Result<Self, ParseError> {
        let mut cursor = Cursor::new(bytes);

        let mut agent_bytes = [0u8; 16];
        cursor.read_exact(&mut agent_bytes)?;
        let agent_id = Uuid::from_bytes(agent_bytes);

        let mut session_bytes = [0u8; 16];
        cursor.read_exact(&mut session_bytes)?;
        let session_id = Uuid::from_bytes(session_bytes);

        let mut region_bytes = [0u8; 16];
        cursor.read_exact(&mut region_bytes)?;
        let region_id = Uuid::from_bytes(region_bytes);

        let x = cursor.read_f32::<LittleEndian>()?;
        let y = cursor.read_f32::<LittleEndian>()?;
        let z = cursor.read_f32::<LittleEndian>()?;
        let position = Vec3 { x, y, z };

        let lx = cursor.read_f32::<LittleEndian>()?;
        let ly = cursor.read_f32::<LittleEndian>()?;
        let lz = cursor.read_f32::<LittleEndian>()?;
        let look_at = Vec3 {
            x: lx,
            y: ly,
            z: lz,
        };

        Ok(TeleportRequest {
            agent_id,
            session_id,
            region_id,
            position,
            look_at,
        })
    }
    fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();

        bytes.extend_from_slice(self.agent_id.as_bytes());
        bytes.extend_from_slice(self.session_id.as_bytes());
        bytes.extend_from_slice(self.region_id.as_bytes());

        bytes.write_f32::<LittleEndian>(self.position.x).unwrap();
        bytes.write_f32::<LittleEndian>(self.position.y).unwrap();
        bytes.write_f32::<LittleEndian>(self.position.z).unwrap();

        bytes.write_f32::<LittleEndian>(self.look_at.x).unwrap();
        bytes.write_f32::<LittleEndian>(self.look_at.y).unwrap();
        bytes.write_f32::<LittleEndian>(self.look_at.z).unwrap();

        bytes
    }
}

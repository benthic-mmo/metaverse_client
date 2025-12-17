use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use glam::Vec3;
use uuid::Uuid;

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
use std::io::{Cursor, Read};

impl Packet {
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

/// add your struct fields here
#[derive(Debug, Clone)]
pub struct TeleportRequest {
    pub agent_id: Uuid,
    pub session_id: Uuid,
    pub region_id: Uuid,
    pub position: Vec3,
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

use byteorder::LittleEndian;
use byteorder::ReadBytesExt;
use byteorder::WriteBytesExt;
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
use std::io::Cursor;
use std::io::Read;

impl Packet {
    pub fn new_agent_throttle(agent_throttle: AgentThrottle) -> Self {
        Packet {
            header: Header {
                id: 81,
                reliable: true,
                zerocoded: true,
                frequency: PacketFrequency::Low,
                ..Default::default()
            },
            body: PacketType::AgentThrottle(Box::new(agent_throttle)),
        }
    }
}

/// add your struct fields here
#[derive(Debug, Clone)]
pub struct AgentThrottle {
    pub agent_id: Uuid,
    pub session_id: Uuid,
    pub circuit_code: u32,
    pub gen_counter: u32,
    pub throttles: ThrottleData,
}

#[derive(Debug, Clone)]
pub struct ThrottleData {
    pub resend: f32,
    pub land: f32,
    pub wind: f32,
    pub cloud: f32,
    pub task: f32,
    pub texture: f32,
    pub asset: f32,
}

impl ThrottleData {
    // Min / Max values
    const MIN_RESEND: f32 = 10_000.0;
    const MAX_RESEND: f32 = 150_000.0;

    const MIN_LAND: f32 = 0.0;
    const MAX_LAND: f32 = 170_000.0;

    const MIN_WIND: f32 = 0.0;
    const MAX_WIND: f32 = 34_000.0;

    const MIN_CLOUD: f32 = 0.0;
    const MAX_CLOUD: f32 = 34_000.0;

    const MIN_TASK: f32 = 4_000.0;
    const MAX_TASK: f32 = 446_000.0 * 3.0;

    const MIN_TEXTURE: f32 = 4_000.0;
    const MAX_TEXTURE: f32 = 446_000.0;

    const MIN_ASSET: f32 = 10_000.0;
    const MAX_ASSET: f32 = 220_000.0;

    pub fn new_total(total: f32) -> Self {
        let mut t = Self {
            resend: total * 0.1,
            land: total * 0.52 / 3.0,
            wind: total * 0.05,
            cloud: total * 0.05,
            task: total * 0.704 / 3.0,
            texture: total * 0.704 / 3.0,
            asset: total * 0.484 / 3.0,
        };
        t.clamp();
        t
    }

    pub fn clamp(&mut self) {
        self.resend = self.resend.clamp(Self::MIN_RESEND, Self::MAX_RESEND);
        self.land = self.land.clamp(Self::MIN_LAND, Self::MAX_LAND);
        self.wind = self.wind.clamp(Self::MIN_WIND, Self::MAX_WIND);
        self.cloud = self.cloud.clamp(Self::MIN_CLOUD, Self::MAX_CLOUD);
        self.task = self.task.clamp(Self::MIN_TASK, Self::MAX_TASK);
        self.texture = self.texture.clamp(Self::MIN_TEXTURE, Self::MAX_TEXTURE);
        self.asset = self.asset.clamp(Self::MIN_ASSET, Self::MAX_ASSET);
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut buf = Vec::with_capacity(7 * 4);
        buf.write_f32::<LittleEndian>(self.resend).unwrap();
        buf.write_f32::<LittleEndian>(self.land).unwrap();
        buf.write_f32::<LittleEndian>(self.wind).unwrap();
        buf.write_f32::<LittleEndian>(self.cloud).unwrap();
        buf.write_f32::<LittleEndian>(self.task).unwrap();
        buf.write_f32::<LittleEndian>(self.texture).unwrap();
        buf.write_f32::<LittleEndian>(self.asset).unwrap();
        buf
    }

    pub fn from_bytes(cursor: &mut Cursor<&[u8]>) -> std::io::Result<Self> {
        let resend = cursor.read_f32::<LittleEndian>()?;
        let land = cursor.read_f32::<LittleEndian>()?;
        let wind = cursor.read_f32::<LittleEndian>()?;
        let cloud = cursor.read_f32::<LittleEndian>()?;
        let task = cursor.read_f32::<LittleEndian>()?;
        let texture = cursor.read_f32::<LittleEndian>()?;
        let asset = cursor.read_f32::<LittleEndian>()?;
        let mut t = Self {
            resend,
            land,
            wind,
            cloud,
            task,
            texture,
            asset,
        };
        t.clamp();
        Ok(t)
    }
}

impl Default for ThrottleData {
    fn default() -> Self {
        Self {
            resend: Self::MAX_RESEND,
            land: Self::MAX_LAND,
            wind: Self::MAX_WIND,
            cloud: Self::MAX_CLOUD,
            task: Self::MAX_TASK,
            texture: Self::MAX_TEXTURE,
            asset: Self::MAX_ASSET,
        }
    }
}

impl PacketData for AgentThrottle {
    fn from_bytes(bytes: &[u8]) -> Result<Self, ParseError> {
        let mut cursor = Cursor::new(bytes);
        let mut agent_id_bytes = [0u8; 16];
        let mut session_id_bytes = [0u8; 16];
        cursor.read_exact(&mut agent_id_bytes)?;
        cursor.read_exact(&mut session_id_bytes)?;
        let agent_id = Uuid::from_bytes(agent_id_bytes);
        let session_id = Uuid::from_bytes(session_id_bytes);
        let circuit_code = cursor.read_u32::<LittleEndian>()?;
        let gen_counter = cursor.read_u32::<LittleEndian>()?;
        let throttles = ThrottleData::from_bytes(&mut cursor)?;
        Ok(Self {
            agent_id,
            session_id,
            circuit_code,
            gen_counter,
            throttles,
        })
    }
    fn to_bytes(&self) -> Vec<u8> {
        let mut buf = Vec::with_capacity(16 + 16 + 4 + 4 + 7 * 4);
        buf.extend_from_slice(self.agent_id.as_bytes());
        buf.extend_from_slice(self.session_id.as_bytes());
        buf.write_u32::<LittleEndian>(self.circuit_code).unwrap();
        buf.write_u32::<LittleEndian>(self.gen_counter).unwrap();
        let throttle_bytes = self.throttles.to_bytes();
        buf.push(throttle_bytes.len() as u8);
        buf.extend_from_slice(&throttle_bytes);
        buf
    }
}

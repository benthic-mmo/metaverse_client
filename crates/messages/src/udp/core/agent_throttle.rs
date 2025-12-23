use crate::errors::ParseError;
use crate::packet::{
    header::{Header, PacketFrequency},
    packet::{Packet, PacketData},
    packet_types::PacketType,
};
use byteorder::LittleEndian;
use byteorder::ReadBytesExt;
use byteorder::WriteBytesExt;
use std::io::Cursor;
use std::io::Read;
use uuid::Uuid;

impl Packet {
    /// create a new agent throttle packet
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

/// This packet is sent to inform the server of the maximum bandwidth the viewer can handle. If
/// this is not sent, the server will throttle to extremely low values, and performance may be
/// impacted.
#[derive(Debug, Clone)]
pub struct AgentThrottle {
    /// the ID of the agent sending the throttle
    pub agent_id: Uuid,
    /// the ID of the session sending the throttle
    pub session_id: Uuid,
    /// the circuit code of the session sending the throttle
    pub circuit_code: u32,
    /// value that tells the server if this throttle is newer than the last one it received.
    /// This is often unused and set to 0.
    pub gen_counter: u32,
    /// throttle values
    pub throttles: ThrottleData,
}

/// Bandwidth limits for individual data categories
#[derive(Debug, Clone)]
pub struct ThrottleData {
    /// maximum bytes per second for resending unacknowledged packets
    pub resend: f32,
    /// maximum bytes per second for sending land patches
    pub land: f32,
    /// maximum bytes per second for sending wind
    pub wind: f32,
    /// maximum bytes per second for sending cloud data
    pub cloud: f32,
    /// maximum bytes per second for object and task data
    pub task: f32,
    /// maximum bytes per second for sending texture data
    pub texture: f32,
    /// maximum bytes per second for sending asset data
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

    /// prevent the values from going over or under the maximum and minimum values
    pub fn clamp(&mut self) {
        self.resend = self.resend.clamp(Self::MIN_RESEND, Self::MAX_RESEND);
        self.land = self.land.clamp(Self::MIN_LAND, Self::MAX_LAND);
        self.wind = self.wind.clamp(Self::MIN_WIND, Self::MAX_WIND);
        self.cloud = self.cloud.clamp(Self::MIN_CLOUD, Self::MAX_CLOUD);
        self.task = self.task.clamp(Self::MIN_TASK, Self::MAX_TASK);
        self.texture = self.texture.clamp(Self::MIN_TEXTURE, Self::MAX_TEXTURE);
        self.asset = self.asset.clamp(Self::MIN_ASSET, Self::MAX_ASSET);
    }

    /// convert throttle data to bytes
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
    /// Decodes `ThrottleData` from a byte stream.
    pub fn from_bytes(cursor: &mut Cursor<&[u8]>) -> Result<ThrottleData, std::io::Error> {
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

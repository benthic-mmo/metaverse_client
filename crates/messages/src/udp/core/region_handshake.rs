use crate::{
    errors::ParseError,
    packet::{
        header::{Header, PacketFrequency},
        packet::{Packet, PacketData},
        packet_types::PacketType,
    },
    utils::agent_access::AgentAccess,
};
use byteorder::{LittleEndian, ReadBytesExt};
use serde::{Deserialize, Serialize};
use std::io::{Cursor, Read};
use uuid::Uuid;

impl Packet {
    /// create a new region handshake packet
    pub fn new_region_handshake(region_handshake: RegionHandshake) -> Self {
        Packet {
            header: Header {
                id: 148,
                frequency: PacketFrequency::Low,
                reliable: true,
                zerocoded: true,
                ..Default::default()
            },
            body: PacketType::RegionHandshake(Box::new(region_handshake)),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
/// The simulator sends this in response to CompleteAgentMovement from the viewer.
/// The viewer responds with RegionHandshakereply, which starts object updates via
/// CoarseLocationUpdate
pub struct RegionHandshake {
    /// undocumented
    pub region_flags: u32,
    /// Access level of the user. Fields like General, Adult, Trial, etc
    pub sim_access: AgentAccess,
    /// Name of the sim
    pub sim_name: String,
    /// UUID of the sim owner
    pub sim_owner: Uuid,
    /// If the user is an estate manager or not
    pub is_estate_manager: bool,
    /// Height of water tiles
    pub water_height: f32,
    /// undocumented
    pub billable_factor: f32,
    /// undocumented
    pub cache_id: Uuid,
    /// undocumented
    pub terrain_base_0: Uuid,
    /// undocumented
    pub terrain_base_1: Uuid,
    /// undocumented
    pub terrain_base_2: Uuid,
    /// undocumented
    pub terrain_base_3: Uuid,
    /// undocumented
    pub terrain_detail_0: Uuid,
    /// undocumented
    pub terrain_detail_1: Uuid,
    /// undocumented
    pub terrain_detail_2: Uuid,
    /// undocumented
    pub terrain_detail_3: Uuid,
    /// undocumented
    pub terrain_start_height_0: f32,
    /// undocumented
    pub terrain_start_height_1: f32,
    /// undocumented
    pub terrain_start_height_2: f32,
    /// undocumented
    pub terrain_start_height_3: f32,
    /// undocumented
    pub terrain_height_range_0: f32,
    /// undocumented
    pub terrain_height_range_1: f32,
    /// undocumented
    pub terrain_height_range_2: f32,
    /// undocumented
    pub terrain_height_range_3: f32,
}

impl PacketData for RegionHandshake {
    /// Convert the RegionHandshake object to bytes
    fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend(&self.region_flags.to_le_bytes());
        bytes.push(self.sim_access.to_bytes());
        bytes.extend(self.sim_name.len().to_le_bytes());
        bytes.extend(self.sim_name.as_bytes());
        bytes.extend(self.sim_owner.as_bytes());
        bytes.push(self.is_estate_manager as u8);
        bytes.extend(&self.water_height.to_le_bytes());
        bytes.extend(&self.billable_factor.to_le_bytes());
        bytes.extend(self.cache_id.as_bytes());
        bytes.extend(self.terrain_base_0.as_bytes());
        bytes.extend(self.terrain_base_1.as_bytes());
        bytes.extend(self.terrain_base_2.as_bytes());
        bytes.extend(self.terrain_base_3.as_bytes());
        bytes.extend(self.terrain_detail_0.as_bytes());
        bytes.extend(self.terrain_detail_1.as_bytes());
        bytes.extend(self.terrain_detail_2.as_bytes());
        bytes.extend(self.terrain_detail_3.as_bytes());
        bytes.extend(&self.terrain_start_height_0.to_le_bytes());
        bytes.extend(&self.terrain_start_height_1.to_le_bytes());
        bytes.extend(&self.terrain_start_height_2.to_le_bytes());
        bytes.extend(&self.terrain_start_height_3.to_le_bytes());
        bytes.extend(&self.terrain_height_range_0.to_le_bytes());
        bytes.extend(&self.terrain_height_range_1.to_le_bytes());
        bytes.extend(&self.terrain_height_range_2.to_le_bytes());
        bytes.extend(&self.terrain_height_range_3.to_le_bytes());
        bytes
    }

    /// Convert bytes to a region handshake object
    fn from_bytes(bytes: &[u8]) -> Result<Self, ParseError> {
        let mut cursor = Cursor::new(bytes);
        let region_flags = cursor.read_u32::<LittleEndian>()?;
        let sim_access_byte = cursor.read_u8()?;
        let sim_access = AgentAccess::from_bytes(&sim_access_byte);

        let sim_name_length = cursor.read_u8()?;
        let mut sim_name_bytes = vec![0u8; sim_name_length as usize];
        cursor.read_exact(&mut sim_name_bytes)?;
        let sim_name = String::from_utf8(sim_name_bytes)?;

        let mut uuid_bytes = [0u8; 16];
        cursor.read_exact(&mut uuid_bytes)?;
        let sim_owner = Uuid::from_slice(&uuid_bytes)?;

        let is_estate_manager = cursor.read_u8()?;

        let water_height = cursor.read_f32::<LittleEndian>()?;

        let billable_factor = cursor.read_f32::<LittleEndian>()?;

        cursor.read_exact(&mut uuid_bytes)?;
        let cache_id = Uuid::from_slice(&uuid_bytes)?;
        cursor.read_exact(&mut uuid_bytes)?;
        let terrain_base_0 = Uuid::from_slice(&uuid_bytes)?;
        cursor.read_exact(&mut uuid_bytes)?;
        let terrain_base_1 = Uuid::from_slice(&uuid_bytes)?;
        cursor.read_exact(&mut uuid_bytes)?;
        let terrain_base_2 = Uuid::from_slice(&uuid_bytes)?;
        cursor.read_exact(&mut uuid_bytes)?;
        let terrain_base_3 = Uuid::from_slice(&uuid_bytes)?;

        cursor.read_exact(&mut uuid_bytes)?;
        let terrain_detail_0 = Uuid::from_slice(&uuid_bytes)?;
        cursor.read_exact(&mut uuid_bytes)?;
        let terrain_detail_1 = Uuid::from_slice(&uuid_bytes)?;
        cursor.read_exact(&mut uuid_bytes)?;
        let terrain_detail_2 = Uuid::from_slice(&uuid_bytes)?;
        cursor.read_exact(&mut uuid_bytes)?;
        let terrain_detail_3 = Uuid::from_slice(&uuid_bytes)?;

        let terrain_start_height_0 = cursor.read_f32::<LittleEndian>()?;
        let terrain_start_height_1 = cursor.read_f32::<LittleEndian>()?;
        let terrain_start_height_2 = cursor.read_f32::<LittleEndian>()?;
        let terrain_start_height_3 = cursor.read_f32::<LittleEndian>()?;

        let terrain_height_range_0 = cursor.read_f32::<LittleEndian>()?;
        let terrain_height_range_1 = cursor.read_f32::<LittleEndian>()?;
        let terrain_height_range_2 = cursor.read_f32::<LittleEndian>()?;
        let terrain_height_range_3 = cursor.read_f32::<LittleEndian>()?;

        Ok(Self {
            region_flags,
            sim_access,
            sim_name,
            sim_owner,
            is_estate_manager: is_estate_manager != 0,
            water_height,
            billable_factor,
            cache_id,
            terrain_base_0,
            terrain_base_1,
            terrain_base_2,
            terrain_base_3,
            terrain_detail_0,
            terrain_detail_1,
            terrain_detail_2,
            terrain_detail_3,
            terrain_start_height_0,
            terrain_start_height_1,
            terrain_start_height_2,
            terrain_start_height_3,
            terrain_height_range_0,
            terrain_height_range_1,
            terrain_height_range_2,
            terrain_height_range_3,
        })
    }
}

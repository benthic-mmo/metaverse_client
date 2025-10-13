use crate::{
    packet::{
        errors::PacketError,
        header::{Header, PacketFrequency},
        packet::{Packet, PacketData},
        packet_types::PacketType,
    },
    utils::agent_access::AgentAccess,
};
use serde::{Deserialize, Serialize};
use std::{
    io::{self, Cursor, Read},
    str::from_utf8,
};
use uuid::Uuid;

impl Packet {
    /// create a new region handshake packet
    pub fn new_region_handshake(region_handshake: RegionHandshake) -> Self {
        Packet {
            header: Header {
                id: 80,
                frequency: PacketFrequency::Low,
                reliable: true,
                sequence_number: 0,
                appended_acks: false,
                zerocoded: true,
                resent: false,
                ack_list: None,
                size: None,
            },
            body: PacketType::RegionHandshake(Box::new(region_handshake)),
        }
    }
}

impl PacketData for RegionHandshake {
    fn from_bytes(bytes: &[u8]) -> Result<Self, PacketError> {
        Ok(serde_json::from_str::<RegionHandshake>(from_utf8(bytes)?)?)
    }
    fn to_bytes(&self) -> Vec<u8> {
        serde_json::to_string(&self).unwrap().into_bytes()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
/// The simulator sends this in response to CompleteAgentMovement from the viewer.
/// The viewer responds with RegionHandshakereply, which starts object updats via
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

impl RegionHandshake {
    /// Convert the RegionHandshake object to bytes
    pub fn to_bytes(&self) -> Vec<u8> {
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
    pub fn from_bytes(bytes: &[u8]) -> io::Result<Self> {
        let mut cursor = Cursor::new(bytes);

        // these region flags are almost certainly super messed up
        let mut region_flags = [0u8; 4];
        cursor.read_exact(&mut region_flags)?;
        let region_flags = u32::from_le_bytes(region_flags);

        // I am not sure what this byte does but it ruins the whole thing if you leave it in :/
        let current_position = cursor.position();
        cursor.set_position(current_position + 1);

        let mut sim_access = [0u8; 1];
        cursor.read_exact(&mut sim_access)?;
        let sim_access = AgentAccess::from_bytes(&sim_access[0]);

        let mut name_len = [0u8; 1];
        cursor.read_exact(&mut name_len)?;
        let name_len = u8::from_le_bytes(name_len) as usize;
        let mut name_bytes = vec![0u8; name_len];
        cursor.read_exact(&mut name_bytes)?;
        let sim_name = String::from_utf8(name_bytes).unwrap();

        let mut uuid_bytes = [0u8; 16];
        cursor.read_exact(&mut uuid_bytes)?;
        let sim_owner = Uuid::from_slice(&uuid_bytes)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

        let mut is_estate_manager = [0u8; 1];
        cursor.read_exact(&mut is_estate_manager)?;

        let mut float_buffer = [0u8; 4];
        cursor.read_exact(&mut float_buffer)?;
        let water_height = f32::from_le_bytes(float_buffer);

        cursor.read_exact(&mut float_buffer)?;
        let billable_factor = f32::from_le_bytes(float_buffer);

        cursor.read_exact(&mut uuid_bytes)?;
        let cache_id = Uuid::from_slice(&uuid_bytes)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

        cursor.read_exact(&mut uuid_bytes)?;
        let terrain_base_0 = Uuid::from_slice(&uuid_bytes)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        cursor.read_exact(&mut uuid_bytes)?;
        let terrain_base_1 = Uuid::from_slice(&uuid_bytes)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        cursor.read_exact(&mut uuid_bytes)?;
        let terrain_base_2 = Uuid::from_slice(&uuid_bytes)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        cursor.read_exact(&mut uuid_bytes)?;
        let terrain_base_3 = Uuid::from_slice(&uuid_bytes)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

        cursor.read_exact(&mut uuid_bytes)?;
        let terrain_detail_0 = Uuid::from_slice(&uuid_bytes)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        cursor.read_exact(&mut uuid_bytes)?;
        let terrain_detail_1 = Uuid::from_slice(&uuid_bytes)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        cursor.read_exact(&mut uuid_bytes)?;
        let terrain_detail_2 = Uuid::from_slice(&uuid_bytes)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        cursor.read_exact(&mut uuid_bytes)?;
        let terrain_detail_3 = Uuid::from_slice(&uuid_bytes)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

        cursor.read_exact(&mut float_buffer)?;
        let terrain_start_height_0 = f32::from_le_bytes(float_buffer);
        cursor.read_exact(&mut float_buffer)?;
        let terrain_start_height_1 = f32::from_le_bytes(float_buffer);
        cursor.read_exact(&mut float_buffer)?;
        let terrain_start_height_2 = f32::from_le_bytes(float_buffer);
        cursor.read_exact(&mut float_buffer)?;
        let terrain_start_height_3 = f32::from_le_bytes(float_buffer);

        cursor.read_exact(&mut float_buffer)?;
        let terrain_height_range_0 = f32::from_le_bytes(float_buffer);
        cursor.read_exact(&mut float_buffer)?;
        let terrain_height_range_1 = f32::from_le_bytes(float_buffer);
        cursor.read_exact(&mut float_buffer)?;
        let terrain_height_range_2 = f32::from_le_bytes(float_buffer);
        cursor.read_exact(&mut float_buffer)?;
        let terrain_height_range_3 = f32::from_le_bytes(float_buffer);

        Ok(Self {
            region_flags,
            sim_access,
            sim_name,
            sim_owner,
            is_estate_manager: is_estate_manager[0] != 0,
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

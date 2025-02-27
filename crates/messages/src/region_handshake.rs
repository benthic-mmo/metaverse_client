use std::io::{self, Cursor, Read};
use uuid::Uuid;

use crate::{
    header::{Header, PacketFrequency},
    packet::Packet,
    packet_types::PacketType,
    utils::agent_access::AgentAccess,
};

impl Packet {
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

#[derive(Debug, Clone)]
pub struct RegionHandshake {
    pub region_info: RegionInfo,
    pub region_info_2: RegionInfo,
    pub region_info_3: RegionInfo,
    pub region_info_4: RegionInfo,
}

impl RegionHandshake {
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend(self.region_info.to_bytes());
        bytes.extend(self.region_info_2.to_bytes());
        bytes.extend(self.region_info_3.to_bytes());
        bytes.extend(self.region_info_4.to_bytes());
        bytes
    }

    pub fn from_bytes(bytes: &[u8]) -> io::Result<Self> {
        let mut cursor = Cursor::new(bytes);

        let region_info = RegionInfo::from_bytes(cursor.get_ref())?;
        cursor.set_position(cursor.position() + region_info.to_bytes().len() as u64);

        let region_info_2 = RegionInfo::from_bytes(cursor.get_ref())?;
        cursor.set_position(cursor.position() + region_info_2.to_bytes().len() as u64);

        let region_info_3 = RegionInfo::from_bytes(cursor.get_ref())?;
        cursor.set_position(cursor.position() + region_info_3.to_bytes().len() as u64);

        let region_info_4 = RegionInfo::from_bytes(cursor.get_ref())?;

        Ok(Self {
            region_info,
            region_info_2,
            region_info_3,
            region_info_4,
        })
    }
}
#[derive(Debug, Clone)]
pub struct RegionInfo {
    pub region_flags: u32,
    pub sim_access: AgentAccess,
    pub sim_name: String,
    pub sim_owner: Uuid,
    pub is_estate_manager: bool,
    pub water_height: f32,
    pub billable_factor: f32,
    pub cache_id: Uuid,
    pub terrain_base_0: Uuid,
    pub terrain_base_1: Uuid,
    pub terrain_base_2: Uuid,
    pub terrain_base_3: Uuid,
    pub terrain_start_height_0: f32,
    pub terrain_start_height_1: f32,
    pub terrain_start_height_2: f32,
    pub terrain_start_height_3: f32,
    pub terrain_height_range_0: f32,
    pub terrain_height_range_1: f32,
    pub terrain_height_range_2: f32,
    pub terrain_height_range_3: f32,
}

impl RegionInfo {
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

#[derive(Debug, Clone)]
pub struct RegionInfo2 {
    pub region_flags_2: u32,
    pub region_owner_id: Uuid,
    pub region_name: String,
    pub terrain_type: u32,
    pub weather_type: u32,
}

impl RegionInfo2 {
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend(&self.region_flags_2.to_le_bytes());
        bytes.extend(self.region_owner_id.as_bytes());
        bytes.extend(self.region_name.len().to_le_bytes());
        bytes.extend(self.region_name.as_bytes());
        bytes.extend(&self.terrain_type.to_le_bytes());
        bytes.extend(&self.weather_type.to_le_bytes());
        bytes
    }

    pub fn from_bytes(bytes: &[u8]) -> io::Result<Self> {
        let mut cursor = Cursor::new(bytes);

        let mut region_flags_2 = [0u8; 4];
        cursor.read_exact(&mut region_flags_2)?;

        let mut uuid_bytes = [0u8; 16];
        cursor.read_exact(&mut uuid_bytes)?;
        let region_owner_id = Uuid::from_slice(&uuid_bytes)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

        let mut name_len = [0u8; 4];
        cursor.read_exact(&mut name_len)?;
        let name_len = u32::from_le_bytes(name_len) as usize;
        let mut name_bytes = vec![0u8; name_len];
        cursor.read_exact(&mut name_bytes)?;
        let region_name = String::from_utf8(name_bytes).unwrap();

        let mut terrain_type = [0u8; 4];
        cursor.read_exact(&mut terrain_type)?;
        let terrain_type = u32::from_le_bytes(terrain_type);

        let mut weather_type = [0u8; 4];
        cursor.read_exact(&mut weather_type)?;
        let weather_type = u32::from_le_bytes(weather_type);

        Ok(Self {
            region_flags_2: u32::from_le_bytes(region_flags_2),
            region_owner_id,
            region_name,
            terrain_type,
            weather_type,
        })
    }
}

#[derive(Debug, Clone)]
pub struct RegionInfo3 {
    pub region_id_3: Uuid,
    pub region_type_3: u8,
    pub region_coordinates: (f32, f32),
    pub region_population: u32,
}

impl RegionInfo3 {
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend(self.region_id_3.as_bytes());
        bytes.push(self.region_type_3);
        bytes.extend(&self.region_coordinates.0.to_le_bytes());
        bytes.extend(&self.region_coordinates.1.to_le_bytes());
        bytes.extend(&self.region_population.to_le_bytes());
        bytes
    }

    pub fn from_bytes(bytes: &[u8]) -> io::Result<Self> {
        let mut cursor = Cursor::new(bytes);

        let mut uuid_bytes = [0u8; 16];
        cursor.read_exact(&mut uuid_bytes)?;
        let region_id_3 = Uuid::from_slice(&uuid_bytes)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

        let mut region_type_3 = [0u8; 1];
        cursor.read_exact(&mut region_type_3)?;

        let mut float_buffer = [0u8; 4];
        cursor.read_exact(&mut float_buffer)?;
        let region_x = f32::from_le_bytes(float_buffer);

        cursor.read_exact(&mut float_buffer)?;
        let region_y = f32::from_le_bytes(float_buffer);

        let mut population_buffer = [0u8; 4];
        cursor.read_exact(&mut population_buffer)?;
        let region_population = u32::from_le_bytes(population_buffer);

        Ok(Self {
            region_id_3,
            region_type_3: region_type_3[0],
            region_coordinates: (region_x, region_y),
            region_population,
        })
    }
}

#[derive(Debug, Clone)]
pub struct RegionInfo4 {
    pub region_flags_4: u16,
    pub owner_name: String,
    pub region_size: u32,
    pub region_capacity: u32,
}

impl RegionInfo4 {
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend(&self.region_flags_4.to_le_bytes());
        bytes.extend(self.owner_name.len().to_le_bytes());
        bytes.extend(self.owner_name.as_bytes());
        bytes.extend(&self.region_size.to_le_bytes());
        bytes.extend(&self.region_capacity.to_le_bytes());
        bytes
    }

    pub fn from_bytes(bytes: &[u8]) -> io::Result<Self> {
        let mut cursor = Cursor::new(bytes);

        let mut region_flags_4 = [0u8; 2];
        cursor.read_exact(&mut region_flags_4)?;

        let mut name_len = [0u8; 4];
        cursor.read_exact(&mut name_len)?;
        let name_len = u32::from_le_bytes(name_len) as usize;
        let mut name_bytes = vec![0u8; name_len];
        cursor.read_exact(&mut name_bytes)?;
        let owner_name = String::from_utf8(name_bytes).unwrap();

        let mut region_size = [0u8; 4];
        cursor.read_exact(&mut region_size)?;
        let region_size = u32::from_le_bytes(region_size);

        let mut region_capacity = [0u8; 4];
        cursor.read_exact(&mut region_capacity)?;
        let region_capacity = u32::from_le_bytes(region_capacity);

        Ok(Self {
            region_flags_4: u16::from_le_bytes(region_flags_4),
            owner_name,
            region_size,
            region_capacity,
        })
    }
}

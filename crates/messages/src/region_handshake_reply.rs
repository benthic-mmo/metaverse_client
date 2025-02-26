use std::io::{self, Cursor, Read};
use uuid::Uuid;

use crate::{
    header::{Header, PacketFrequency},
    packet::{Packet, PacketData},
    packet_types::PacketType,
};

impl Packet {
    pub fn new_region_handshake_reply(region_handshake_reply: RegionHandshakeReply) -> Self {
        Packet {
            header: Header {
                id: 149,
                reliable: false,
                resent: false,
                zerocoded: true,
                appended_acks: false,
                sequence_number: 0,
                frequency: PacketFrequency::Low,
                ack_list: None,
                size: None,
            },
            body: PacketType::RegionHandshakeReply(Box::new(region_handshake_reply)),
        }
    }
}

#[derive(Debug, Clone)]
pub struct RegionHandshakeReply {
    pub agent_data: AgentData,
    pub region_info: ReplyRegionInfo,
}

#[derive(Debug, Clone)]
pub struct AgentData {
    pub agent_id: Uuid,
    pub session_id: Uuid,
}
impl AgentData {
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend(self.agent_id.as_bytes());
        bytes.extend(self.session_id.as_bytes());
        bytes
    }

    pub fn from_bytes(bytes: &[u8]) -> io::Result<Self> {
        if bytes.len() != 32 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Invalid byte length for AgentData",
            ));
        }

        let agent_id = Uuid::from_slice(&bytes[0..16])
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        let session_id = Uuid::from_slice(&bytes[16..32])
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

        Ok(AgentData {
            agent_id,
            session_id,
        })
    }
}

#[derive(Debug, Clone)]
pub struct ReplyRegionInfo {
    pub flags: u32,
}
impl ReplyRegionInfo {
    pub fn to_bytes(&self) -> Vec<u8> {
        self.flags.to_le_bytes().to_vec()
    }

    pub fn from_bytes(bytes: &[u8]) -> io::Result<Self> {
        if bytes.len() != 4 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Invalid byte length for RegionInfo",
            ));
        }

        let flags = u32::from_le_bytes(bytes.try_into().map_err(|_| {
            io::Error::new(io::ErrorKind::InvalidData, "Failed to convert bytes to u32")
        })?);

        Ok(ReplyRegionInfo { flags })
    }
}

impl PacketData for RegionHandshakeReply {
    fn from_bytes(bytes: &[u8]) -> io::Result<Self> {
        let mut cursor = Cursor::new(bytes);
        let mut agent_data_bytes = [0u8; 32]; // 16 bytes for agent_id + 16 bytes for session_id
        cursor.read_exact(&mut agent_data_bytes)?;
        let agent_data = AgentData::from_bytes(&agent_data_bytes)?;

        let mut region_info_bytes = [0u8; 4]; // 4 bytes for flags
        cursor.read_exact(&mut region_info_bytes)?;
        let region_info = ReplyRegionInfo::from_bytes(&region_info_bytes)?;

        Ok(RegionHandshakeReply {
            agent_data,
            region_info,
        })
    }
    fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend(self.agent_data.to_bytes());
        bytes.extend(self.region_info.to_bytes());
        bytes
    }
}

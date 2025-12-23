use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use std::io::{Cursor, Read};
use uuid::Uuid;

use crate::errors::ParseError;
use crate::packet::{
    header::{Header, PacketFrequency},
    packet::{Packet, PacketData},
    packet_types::PacketType,
};

impl Packet {
    /// create a new request multiple objects packet
    pub fn new_request_multiple_objects(request_multiple_objects: RequestMultipleObjects) -> Self {
        Packet {
            header: Header {
                id: 3,
                reliable: true,
                zerocoded: true,
                frequency: PacketFrequency::Medium,
                ..Default::default()
            },
            body: PacketType::RequestMultipleObjects(Box::new(request_multiple_objects)),
        }
    }
}

#[derive(Debug, Clone)]
/// Struct to request multiple objects from the server.
/// Uses the u32 local ID retrieved from ObjectUpdate packets to request full data.
pub struct RequestMultipleObjects {
    /// agent ID requesting objects
    pub agent_id: Uuid,
    /// session ID requesting objects
    pub session_id: Uuid,
    /// Tells the server what type of data is missing.
    pub requests: Vec<(CacheMissType, u32)>,
}

#[derive(Debug, Clone, Copy)]
/// Type that defines which type of data is missing
pub enum CacheMissType {
    /// default request
    Normal = 0,
    /// request attachment data
    Attachment = 1,
    /// custom request
    Other = 2,
}

impl From<u8> for CacheMissType {
    fn from(value: u8) -> Self {
        match value {
            0 => CacheMissType::Normal,
            1 => CacheMissType::Attachment,
            _ => CacheMissType::Other,
        }
    }
}

impl From<CacheMissType> for u8 {
    fn from(value: CacheMissType) -> Self {
        value as u8
    }
}

impl PacketData for RequestMultipleObjects {
    fn from_bytes(bytes: &[u8]) -> Result<Self, ParseError> {
        let mut cursor = Cursor::new(bytes);

        let mut agent_bytes = [0u8; 16];
        cursor.read_exact(&mut agent_bytes)?;
        let agent_id = Uuid::from_bytes(agent_bytes);

        let mut session_bytes = [0u8; 16];
        cursor.read_exact(&mut session_bytes)?;
        let session_id = Uuid::from_bytes(session_bytes);

        let request_len = cursor.read_u8()?;

        let mut requests = Vec::new();
        for _ in 0..request_len {
            let cache_miss_type_raw = cursor.read_u8()?;
            let cache_miss_type = CacheMissType::from(cache_miss_type_raw);
            let id = cursor.read_u32::<LittleEndian>()?;
            requests.push((cache_miss_type, id));
        }

        Ok(RequestMultipleObjects {
            agent_id,
            session_id,
            requests,
        })
    }

    fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new(); // 32 bytes for UUIDs + 5 per request
        bytes.extend_from_slice(self.agent_id.as_bytes());
        bytes.extend_from_slice(self.session_id.as_bytes());
        bytes.extend_from_slice(&[(self.requests.len() as u8)]);
        for (cache_miss_type, id) in &self.requests {
            bytes.push(u8::from(*cache_miss_type));
            bytes.write_u32::<LittleEndian>(*id).unwrap();
        }

        bytes
    }
}

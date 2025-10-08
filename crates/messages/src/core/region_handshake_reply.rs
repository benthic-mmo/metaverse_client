use crate::packet::{
    errors::PacketError,
    header::{Header, PacketFrequency},
    packet::{Packet, PacketData},
    packet_types::PacketType,
};
use std::io::{self, Cursor, Read};
use uuid::Uuid;

impl Packet {
    /// create a new region handshake reply packet
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
/// Sent in response to a Regionhandshake, which finishes the handshake
pub struct RegionHandshakeReply {
    /// The user's agent ID
    pub agent_id: Uuid,
    /// The viewer's session ID
    pub session_id: Uuid,
    /// Region info flags in the repy.
    pub flags: u32,
}

impl PacketData for RegionHandshakeReply {
    fn from_bytes(bytes: &[u8]) -> Result<Self, PacketError> {
        let mut cursor = Cursor::new(bytes);

        let mut agent_id_bytes = [0u8; 16];
        cursor.read_exact(&mut agent_id_bytes)?;
        let agent_id = Uuid::from_slice(&agent_id_bytes)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        let mut session_id_bytes = [0u8; 16];

        cursor.read_exact(&mut session_id_bytes)?;
        let session_id = Uuid::from_slice(&session_id_bytes)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

        let mut region_info_bytes = [0u8; 4]; // 4 bytes for flags
        cursor.read_exact(&mut region_info_bytes)?;

        let flags = u32::from_le_bytes(bytes.try_into().map_err(|_| {
            io::Error::new(io::ErrorKind::InvalidData, "Failed to convert bytes to u32")
        })?);

        Ok(RegionHandshakeReply {
            agent_id,
            session_id,
            flags,
        })
    }
    fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend(self.agent_id.as_bytes());
        bytes.extend(self.session_id.as_bytes());
        bytes.extend(self.flags.to_le_bytes().to_vec());
        bytes
    }
}

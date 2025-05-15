use std::io::{self, Cursor, Read};
use uuid::Uuid;

use crate::packet::{
    header::{Header, PacketFrequency},
    packet::{Packet, PacketData},
    packet_types::PacketType,
};

impl Packet {
    /// create a new logout request 
    pub fn new_logout_request(logout_request: LogoutRequest) -> Self {
        Packet {
            header: Header {
                id: 252,
                reliable: true,
                resent: false,
                zerocoded: false,
                appended_acks: false,
                sequence_number: 0,
                frequency: PacketFrequency::Low,
                ack_list: None,
                size: None,
            },
            body: PacketType::LogoutRequest(Box::new(logout_request)),
        }
    }
}

/// add your struct fields here
#[derive(Debug, Clone)]
pub struct LogoutRequest {
    /// the agent ID of the user to logut 
    pub agent_id: Uuid,
    /// the session ID to log out 
    pub session_id: Uuid,
}

impl PacketData for LogoutRequest {
    fn from_bytes(bytes: &[u8]) -> io::Result<Self> {
        let mut cursor = Cursor::new(bytes);

        let mut id_bytes = [0u8; 16];
        cursor.read_exact(&mut id_bytes)?;
        let agent_id = Uuid::from_bytes(id_bytes);

        let mut session_bytes = [0u8; 16];
        cursor.read_exact(&mut session_bytes)?;
        let session_id = Uuid::from_bytes(session_bytes);
        Ok(LogoutRequest {
            agent_id,
            session_id,
        })
    }
    fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(self.agent_id.as_bytes());
        bytes.extend_from_slice(self.session_id.as_bytes());
        bytes
    }
}

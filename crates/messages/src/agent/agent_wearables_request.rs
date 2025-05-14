use crate::packet::{
    header::{Header, PacketFrequency},
    packet::{Packet, PacketData},
    packet_types::PacketType,
};
use std::io::{self, Cursor, Read};
use uuid::Uuid;

impl Packet {
    pub fn new_agent_wearables_request(agent_wearables_request: AgentWearablesRequest) -> Self {
        Packet {
            header: Header {
                id: 381,
                reliable: false,
                resent: false,
                zerocoded: false,
                appended_acks: false,
                sequence_number: 0,
                frequency: PacketFrequency::Low,
                ack_list: None,
                size: None,
            },
            body: PacketType::AgentWearablesRequest(Box::new(agent_wearables_request)),
        }
    }
}

#[derive(Debug, Clone)]
pub struct AgentWearablesRequest {
    agent_id: Uuid,
    session_id: Uuid,
}

impl PacketData for AgentWearablesRequest {
    fn from_bytes(bytes: &[u8]) -> io::Result<Self> {
        let mut cursor = Cursor::new(bytes);

        let mut id_bytes = [0u8; 16];
        cursor.read_exact(&mut id_bytes)?;
        let agent_id = Uuid::from_bytes(id_bytes);

        let mut session_bytes = [0u8; 16];
        cursor.read_exact(&mut session_bytes)?;
        let session_id = Uuid::from_bytes(session_bytes);

        Ok(AgentWearablesRequest {
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

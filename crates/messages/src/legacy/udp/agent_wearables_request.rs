use crate::{
    errors::ParseError,
    packet::{
        header::{Header, PacketFrequency},
        packet::{Packet, PacketData},
        packet_types::PacketType,
    },
};
use std::io::{Cursor, Read};
use uuid::Uuid;

impl Packet {
    /// Create a new agent wearables request
    pub fn new_agent_wearables_request(agent_wearables_request: AgentWearablesRequest) -> Self {
        Packet {
            header: Header {
                id: 381,
                reliable: false,
                zerocoded: false,
                frequency: PacketFrequency::Low,
                ..Default::default()
            },
            body: PacketType::AgentWearablesRequest(Box::new(agent_wearables_request)),
        }
    }
}

#[derive(Debug, Clone)]
/// This is used for requesting the wearables from a user. This is actually legacy code. This has
/// been replaced by the FetchLibDescendents endpoint. When you receive an object update from
/// another user, you send the FetchLibDescendents endpoint a post request asking for its
/// currently_worn folder. The server will reply with an AgentWearablesUpdate, which will usually
/// contain no useful data.
pub struct AgentWearablesRequest {
    /// The agent ID of the user you want to request wearables for
    pub agent_id: Uuid,
    /// your current session ID
    pub session_id: Uuid,
}

impl PacketData for AgentWearablesRequest {
    fn from_bytes(bytes: &[u8]) -> Result<Self, ParseError> {
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

use crate::{
    errors::ParseError,
    packet::{
        header::{Header, PacketFrequency},
        packet_protocol::{Packet, PacketData},
        packet_types::PacketType,
    },
    parse,
};
use uuid::Uuid;

impl Packet {
    /// create a new complete agent movement packet
    pub fn new_complete_agent_movement(
        complete_agent_movement_data: CompleteAgentMovementData,
    ) -> Self {
        Packet {
            header: Header {
                id: 249,
                frequency: PacketFrequency::Low,
                reliable: false,
                zerocoded: false,
                ..Default::default()
            },
            body: PacketType::CompleteAgentMovementData(Box::new(complete_agent_movement_data)),
        }
    }
}

#[derive(Debug, Clone)]
/// this packet completes agent movement and allows the user presence to appear in the server.
pub struct CompleteAgentMovementData {
    /// ID of the user agent
    pub agent_id: Uuid,
    /// ID of the session agent
    pub session_id: Uuid,
    /// circuit code received by the server for trusted packets  
    pub circuit_code: u32,
}

impl PacketData for CompleteAgentMovementData {
    fn from_bytes(bytes: &[u8]) -> Result<Self, ParseError> {
        let circuit_code = u32::from_le_bytes(parse!(bytes[0..4].try_into())?);
        let session_id = parse!(Uuid::from_slice(&bytes[4..20]))?;
        let agent_id = parse!(Uuid::from_slice(&bytes[20..36]))?;

        Ok(CompleteAgentMovementData {
            agent_id,
            session_id,
            circuit_code,
        })
    }
    fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(36);
        bytes.extend_from_slice(self.agent_id.as_bytes());
        bytes.extend(self.session_id.as_bytes());
        bytes.extend(self.circuit_code.to_le_bytes());
        bytes
    }
}

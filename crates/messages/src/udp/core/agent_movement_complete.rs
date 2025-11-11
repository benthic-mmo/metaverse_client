use crate::errors::ParseError;
use crate::packet::{
    header::{Header, PacketFrequency},
    packet::{Packet, PacketData},
    packet_types::PacketType,
};
use std::io::Cursor;

impl Packet {
    /// create a new agent movement complete packet
    pub fn new_agent_movement_complete(agent_movement_complete: AgentMovementComplete) -> Self {
        Packet {
            header: Header {
                id: 250,
                reliable: true,
                resent: false,
                zerocoded: false,
                appended_acks: false,
                sequence_number: 0,
                frequency: PacketFrequency::Low,
                ack_list: None,
                size: None,
            },
            body: PacketType::AgentMovementComplete(Box::new(agent_movement_complete)),
        }
    }
}

#[derive(Debug, Clone)]
/// TODO: UNIMPLEMENTED
pub struct AgentMovementComplete {}

impl PacketData for AgentMovementComplete {
    fn from_bytes(bytes: &[u8]) -> Result<Self, ParseError> {
        let mut _cursor = Cursor::new(bytes);
        Ok(AgentMovementComplete {})
    }
    fn to_bytes(&self) -> Vec<u8> {
        
        Vec::new()
    }
}

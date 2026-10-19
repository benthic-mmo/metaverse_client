use crate::errors::ParseError;
use crate::packet::{
    header::{Header, PacketFrequency},
    packet_protocol::{Packet, PacketData},
    packet_types::PacketType,
};
use std::io::Cursor;

impl Packet {
    /// TODO: this is a stub
    pub fn new_agent_data_update(agent_data_update: AgentDataUpdate) -> Self {
        Packet {
            header: Header {
                id: 387,
                reliable: false,
                zerocoded: false,
                frequency: PacketFrequency::Low,
                ..Default::default()
            },
            body: PacketType::AgentDataUpdate(Box::new(agent_data_update)),
        }
    }
}

#[derive(Debug, Clone)]
/// TODO: this is a stub
pub struct AgentDataUpdate {}

impl PacketData for AgentDataUpdate {
    fn from_bytes(bytes: &[u8]) -> Result<Self, ParseError> {
        let _cursor = Cursor::new(bytes);
        Ok(AgentDataUpdate {})
    }
    fn to_bytes(&self) -> Vec<u8> {
        Vec::new()
    }
}

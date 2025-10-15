use serde::{Deserialize, Serialize};

use crate::{
    errors::ParseError,
    packet::{
        header::{Header, PacketFrequency},
        message::UIMessage,
        packet::{Packet, PacketData},
        packet_types::PacketType,
    },
};

impl Packet {
    /// create a new disable simulator packet
    pub fn new_disable_simulator(disable_simulator: DisableSimulator) -> Self {
        Packet {
            header: Header {
                id: 152,
                reliable: true,
                resent: false,
                zerocoded: false,
                appended_acks: false,
                sequence_number: 0,
                frequency: PacketFrequency::Low,
                ack_list: None,
                size: None,
            },
            body: PacketType::DisableSimulator(Box::new(disable_simulator)),
        }
    }
}

impl UIMessage {
    /// create a new UI message to allow the client to inform the server of disconnects
    pub fn new_disable_simulator() -> Self {
        UIMessage::DisableSimulator(DisableSimulator {})
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
/// the disable simulator struct. Intentionally Contains no values.
pub struct DisableSimulator {}

impl PacketData for DisableSimulator {
    fn from_bytes(_: &[u8]) -> Result<Self, ParseError> {
        Ok(DisableSimulator {})
    }
    fn to_bytes(&self) -> Vec<u8> {
        vec![]
    }
}

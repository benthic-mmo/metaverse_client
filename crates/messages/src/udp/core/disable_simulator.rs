use serde::{Deserialize, Serialize};

use crate::{
    errors::ParseError,
    packet::{
        header::{Header, PacketFrequency},
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
                zerocoded: false,
                frequency: PacketFrequency::Low,
                ..Default::default()
            },
            body: PacketType::DisableSimulator(Box::new(disable_simulator)),
        }
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

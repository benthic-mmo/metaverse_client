use crate::packet::{
    header::{Header, PacketFrequency},
    packet::{Packet, PacketData},
    packet_types::PacketType,
};
use std::io;

impl Packet {
    /// create a new disable simulator packet
    pub fn new_disable_simulator(disable_simulator: DisableSimulator) -> Self {
        Packet {
            header: Header {
                id: 152,
                reliable: false,
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

#[derive(Debug, Clone)]
/// the disable simulator struct. Intentionally Contains no values.
pub struct DisableSimulator {}

impl PacketData for DisableSimulator {
    fn from_bytes(_: &[u8]) -> io::Result<Self> {
        Ok(DisableSimulator {})
    }
    fn to_bytes(&self) -> Vec<u8> {
        vec![]
    }
}

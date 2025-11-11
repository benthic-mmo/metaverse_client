use crate::errors::ParseError;
use crate::packet::{
    header::{Header, PacketFrequency},
    packet::{Packet, PacketData},
    packet_types::PacketType,
};
use std::io::Cursor;

impl Packet {
    /// create a new sim stats packet
    pub fn new_sim_stats(sim_stats: SimStats) -> Self {
        Packet {
            header: Header {
                id: 140,
                reliable: true,
                resent: false,
                zerocoded: false,
                appended_acks: false,
                sequence_number: 0,
                frequency: PacketFrequency::Low,
                ack_list: None,
                size: None,
            },
            body: PacketType::SimStats(Box::new(sim_stats)),
        }
    }
}

#[derive(Debug, Clone)]
/// TODO: UNIMPLEMENTED
pub struct SimStats {}

impl PacketData for SimStats {
    fn from_bytes(bytes: &[u8]) -> Result<Self, ParseError> {
        let mut _cursor = Cursor::new(bytes);
        Ok(SimStats {})
    }
    fn to_bytes(&self) -> Vec<u8> {
        
        Vec::new()
    }
}

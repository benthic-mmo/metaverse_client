use crate::errors::ParseError;
use crate::packet::{
    header::{Header, PacketFrequency},
    packet::{Packet, PacketData},
    packet_types::PacketType,
};
/// this is a file for easily creating a new packet.
/// Simply copy this and fill in the data to create a new packet
/// *local_name*    is something like "region_handshake"
/// *PacketName*    is the name of the packet like "RegionHandshake"
/// *id*            is the ID of the packet
///
use std::io::Cursor;

impl Packet {
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

/// add your struct fields here
#[derive(Debug, Clone)]
pub struct SimStats {}

impl PacketData for SimStats {
    fn from_bytes(bytes: &[u8]) -> Result<Self, ParseError> {
        let mut cursor = Cursor::new(bytes);
        // handle from bytes
        Ok(SimStats{
            // Struct fields 
        })
    }
    fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        // push your data into the new vector
        bytes
    }
}

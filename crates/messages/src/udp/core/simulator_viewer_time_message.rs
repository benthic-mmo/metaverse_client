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
    pub fn new_simulator_viewer_time_message(
        simulator_viewer_time_message: SimulatorViewerTimeMessage,
    ) -> Self {
        Packet {
            header: Header {
                id: 150,
                reliable: true,
                zerocoded: false,
                frequency: PacketFrequency::Low,
                ..Default::default()
            },
            body: PacketType::SimulatorViewerTimeMessage(Box::new(simulator_viewer_time_message)),
        }
    }
}

/// add your struct fields here
#[derive(Debug, Clone)]
pub struct SimulatorViewerTimeMessage {}

impl PacketData for SimulatorViewerTimeMessage {
    fn from_bytes(bytes: &[u8]) -> Result<Self, ParseError> {
        let mut cursor = Cursor::new(bytes);
        // handle from bytes
        Ok(SimulatorViewerTimeMessage{
            // Struct fields 
        })
    }
    fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        // push your data into the new vector
        bytes
    }
}

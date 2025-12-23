use crate::errors::ParseError;
use crate::packet::{
    header::{Header, PacketFrequency},
    packet::{Packet, PacketData},
    packet_types::PacketType,
};
use std::io::Cursor;

impl Packet {
    /// create a new simulator viewer time packet
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

/// TODO: unimplemented
#[derive(Debug, Clone)]
pub struct SimulatorViewerTimeMessage {}

impl PacketData for SimulatorViewerTimeMessage {
    fn from_bytes(bytes: &[u8]) -> Result<Self, ParseError> {
        let _cursor = Cursor::new(bytes);
        Ok(SimulatorViewerTimeMessage {})
    }
    fn to_bytes(&self) -> Vec<u8> {
        Vec::new()
    }
}

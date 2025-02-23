use super::packet::PacketData;
use std::io;

// ID: 152
// Frequency: Low

#[derive(Debug, Clone)]
pub struct DisableSimulator {}

impl PacketData for DisableSimulator {
    fn from_bytes(_: &[u8]) -> io::Result<Self> {
        Ok(DisableSimulator {})
    }
    fn to_bytes(&self) -> Vec<u8> {
        vec![]
    }
}

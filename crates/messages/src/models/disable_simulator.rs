use super::packet::PacketData;
use std::io;

#[derive(Debug)]
pub struct DisableSimulator {}

impl PacketData for DisableSimulator {
    fn from_bytes(_: &[u8]) -> io::Result<Self> {
        Ok(DisableSimulator {})
    }
    fn to_bytes(&self) -> Vec<u8> {
        vec![]
    }
}

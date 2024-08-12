use super::{
    disable_simulator::DisableSimulator, packet::PacketData, packet_ack::PacketAck,
    use_circuit_code::CircuitCodeData,
};
use std::io;

#[derive(Debug)]
pub enum PacketType {
    CircuitCode(CircuitCodeData),
    DisableSimulator(DisableSimulator),
    PacketAck(PacketAck),
}

impl PacketType {
    pub fn from_id(id: u16, bytes: &[u8]) -> io::Result<Self> {
        match id {
            3 => Ok(PacketType::CircuitCode(CircuitCodeData::from_bytes(bytes)?)),
            152 => Ok(PacketType::DisableSimulator(DisableSimulator::from_bytes(
                bytes,
            )?)),
            65531 => Ok(PacketType::PacketAck(PacketAck::from_bytes(bytes)?)),
            _ => Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Unknown packet ID",
            )),
        }
    }
}

use super::{
    circuit_code::CircuitCodeData, coarse_location_update::CoarseLocationUpdate,
    disable_simulator::DisableSimulator, header::PacketFrequency, packet::PacketData,
    packet_ack::PacketAck,
};
use metaverse_utils::IntoBoxed;
use std::io;

#[derive(Debug, IntoBoxed)]
pub enum PacketType {
    CircuitCode(Box<dyn PacketData>),
    DisableSimulator(Box<dyn PacketData>),
    PacketAck(Box<dyn PacketData>),
    CoarseLocationUpdate(Box<dyn PacketData>),
}

impl PacketType {
    pub fn from_id(id: u16, frequency: PacketFrequency, bytes: &[u8]) -> io::Result<Self> {
        // the packets are organized by frquency.
        // I really don't like it, but there's nothing I can do about it
        // I will eventually organize these by type
        // Acknowledgements,
        // Requests,
        // Commands,
        // Errors,
        // Data.
        match frequency {
            PacketFrequency::High => match id {
                id => Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!("Unknown packet ID: {}, frequency: {}", id, frequency),
                )),
            },
            PacketFrequency::Medium => match id {
                6 => Ok(PacketType::CoarseLocationUpdate(Box::new(
                    CoarseLocationUpdate::from_bytes(bytes)?,
                ))),
                id => Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!("Unknown packet ID: {}, frequency: {}", id, frequency),
                )),
            },
            PacketFrequency::Low => match id {
                3 => Ok(PacketType::CircuitCode(Box::new(
                    CircuitCodeData::from_bytes(bytes)?,
                ))),
                152 => Ok(PacketType::DisableSimulator(Box::new(
                    DisableSimulator::from_bytes(bytes)?,
                ))),
                65531 => Ok(PacketType::PacketAck(Box::new(PacketAck::from_bytes(
                    bytes,
                )?))),
                id => Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!("Unknown packet ID: {}, frequency: {}", id, frequency),
                )),
            },
        }
    }
}

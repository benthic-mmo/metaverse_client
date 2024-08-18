use super::agent_update::AgentUpdate;
use super::chat_from_simulator::ChatFromSimulator;
use super::complete_agent_movement::CompleteAgentMovementData;
use super::{
    circuit_code::CircuitCodeData, coarse_location_update::CoarseLocationUpdate,
    disable_simulator::DisableSimulator, header::PacketFrequency, packet::PacketData,
    packet_ack::PacketAck,
};
use metaverse_utils::IntoArc;
use std::io;
use std::sync::Arc;

// IntoArc provides a macro that allows all of these to be contained within arcs
// this is reqired for PacketData to be object safe
// I'm doing it this way because writing them all out is tedious,
// and I want to have as few packet definitions as possible in this project
#[derive(Debug, IntoArc)]
pub enum PacketType {
    CircuitCode(Box<dyn PacketData>),
    DisableSimulator(Box<dyn PacketData>),
    PacketAck(Box<dyn PacketData>),
    CoarseLocationUpdate(Box<dyn PacketData>),
    CompleteAgentMovementData(Box<dyn PacketData>),
    AgentUpdate(Box<dyn PacketData>),
    ChatFromSimulator(Box<dyn PacketData>),
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
                4 => Ok(PacketType::AgentUpdate(Box::new(AgentUpdate::from_bytes(
                    bytes,
                )?))),
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
                249 => Ok(PacketType::CompleteAgentMovementData(Box::new(
                    CompleteAgentMovementData::from_bytes(bytes)?,
                ))),
                139 => Ok(PacketType::ChatFromSimulator(Box::new(
                    ChatFromSimulator::from_bytes(bytes)?,
                ))),
                id => Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!("Unknown packet ID: {}, frequency: {}", id, frequency),
                )),
            },
            PacketFrequency::Fixed => match id {
                251 => Ok(PacketType::PacketAck(Box::new(PacketAck::from_bytes(
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

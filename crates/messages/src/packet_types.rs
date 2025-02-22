use futures::future::BoxFuture;

use crate::errors::SessionError;
use crate::login_system::login::Login;
use crate::login_system::login_response::LoginResponse;
use crate::packet::MessageType;
use crate::ui_events::UiEventTypes;

use super::agent_update::AgentUpdate;
use super::chat_from_simulator::ChatFromSimulator;
use super::chat_from_viewer::ChatFromViewer;
use super::complete_agent_movement::CompleteAgentMovementData;
use super::{
    circuit_code::CircuitCodeData, coarse_location_update::CoarseLocationUpdate,
    disable_simulator::DisableSimulator, header::PacketFrequency, packet::PacketData,
    packet_ack::PacketAck,
};
use std::io;

// IntoArc provides a macro that allows all of these to be contained within arcs
// this is reqired for PacketData to be object safe
// I'm doing it this way because writing them all out is tedious,
// and I want to have as few packet definitions as possible in this project
//#[derive(Debug, IntoArc)]
#[derive(Debug, Clone)]
pub enum PacketType {
    CircuitCode(Box<CircuitCodeData>),
    DisableSimulator(Box<DisableSimulator>),
    PacketAck(Box<PacketAck>),
    CoarseLocationUpdate(Box<CoarseLocationUpdate>),
    CompleteAgentMovementData(Box<CompleteAgentMovementData>),
    AgentUpdate(Box<AgentUpdate>),
    ChatFromSimulator(Box<ChatFromSimulator>),
    ChatFromViewer(Box<ChatFromViewer>),
    // these do not exist in the packet spec! Used as utilities for communicating with server and
    // client. 
    Login(Box<Login>),
    LoginResponse(Box<LoginResponse>),
    Error(Box<SessionError>),
}
impl PacketType {
    pub fn message_type(&self) -> MessageType {
        match self {
            PacketType::ChatFromViewer(_) => MessageType::Event,
            PacketType::CoarseLocationUpdate(_) => MessageType::Event,
            PacketType::DisableSimulator(_) => MessageType::Event,

            PacketType::AgentUpdate(_) => MessageType::Outgoing,
            PacketType::CompleteAgentMovementData(_) => MessageType::Outgoing,
            PacketType::ChatFromSimulator(_) => MessageType::Outgoing,
            PacketType::CircuitCode(_) => MessageType::Outgoing,

            PacketType::PacketAck(_) => MessageType::Acknowledgment,

            PacketType::Login(_) => MessageType::Login,
            PacketType::LoginResponse(_) => MessageType::Login,
            PacketType::Error(_) => MessageType::Error,
        }
    }
    pub fn ui_event(&self) -> UiEventTypes {
        match self {
            PacketType::ChatFromViewer(_) => UiEventTypes::ChatEvent,
            PacketType::CoarseLocationUpdate(_) => UiEventTypes::CoarseLocationUpdateEvent,
            PacketType::DisableSimulator(_) => UiEventTypes::DisableSimulatorEvent,
            _ => UiEventTypes::None,
        }
    }
    pub fn to_bytes(&self) -> Vec<u8> {
        match self {
            PacketType::CircuitCode(data) => data.to_bytes(),
            PacketType::DisableSimulator(data) => data.to_bytes(),
            PacketType::PacketAck(data) => data.to_bytes(),
            PacketType::CoarseLocationUpdate(data) => data.to_bytes(),
            PacketType::CompleteAgentMovementData(data) => data.to_bytes(),
            PacketType::AgentUpdate(data) => data.to_bytes(),
            PacketType::ChatFromSimulator(data) => data.to_bytes(),
            PacketType::ChatFromViewer(data) => data.to_bytes(),
            PacketType::Login(data) => data.to_bytes(),
            PacketType::LoginResponse(_) => Vec::new(),
            PacketType::Error(data) => data.to_bytes(),
        }
    }

    pub fn on_receive(&self) -> BoxFuture<'static, ()> {
        match self {
            PacketType::CircuitCode(data) => data.on_receive(),
            PacketType::DisableSimulator(data) => data.on_receive(),
            PacketType::PacketAck(data) => data.on_receive(),
            PacketType::CoarseLocationUpdate(data) => data.on_receive(),
            PacketType::CompleteAgentMovementData(data) => data.on_receive(),
            PacketType::AgentUpdate(data) => data.on_receive(),
            PacketType::ChatFromSimulator(data) => data.on_receive(),
            PacketType::ChatFromViewer(data) => data.on_receive(),
            PacketType::Login(data) => data.on_receive(),
            PacketType::LoginResponse(_) => {Box::pin(async move {
                println!("LoginResponse on_recieve not yet implemented");
            })}
            PacketType::Error(_) => {Box::pin(async move {
                println!("Error on_recieve not yet implemented");
            })}
        }
    }
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
                80 => Ok(PacketType::ChatFromViewer(Box::new(
                    ChatFromViewer::from_bytes(bytes)?,
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
                66 => Ok(PacketType::Login(Box::new(Login::from_bytes(bytes)?))),

                id => Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!("Unknown packet ID: {}, frequency: {}", id, frequency),
                )),
            },
        }
    }
}

use std::fmt::Debug;
use std::io;

use super::header::PacketFrequency;
use crate::agent::agent_wearables_request::AgentWearablesRequest;
use crate::agent::agent_wearables_update::AgentWearablesUpdate;
use crate::agent::avatar_appearance::AvatarAppearance;
use crate::core::object_update::ObjectUpdate;
use crate::login::logout_request::LogoutRequest;
use crate::packet::packet::PacketData;
use crate::ui::errors::SessionError;
use crate::{
    agent::{agent_update::AgentUpdate, coarse_location_update::CoarseLocationUpdate},
    chat::{chat_from_simulator::ChatFromSimulator, chat_from_viewer::ChatFromViewer},
    core::{
        complete_ping_check::CompletePingCheck, disable_simulator::DisableSimulator,
        packet_ack::PacketAck, region_handshake::RegionHandshake,
        region_handshake_reply::RegionHandshakeReply, start_ping_check::StartPingCheck,
    },
    environment::layer_data::LayerData,
    login::{
        circuit_code::CircuitCodeData, complete_agent_movement::CompleteAgentMovementData,
        login_response::LoginResponse, login_xmlrpc::Login,
    },
    ui::{mesh_update::MeshUpdate, ui_events::UiEventTypes},
};

#[derive(Debug, Clone)]
/// Types of all of the packets, to allow for deserialization
pub enum PacketType {
    /// CircuitCode packet
    CircuitCode(Box<CircuitCodeData>),
    /// Disable Simulator packet
    DisableSimulator(Box<DisableSimulator>),
    /// PaceketAck packet
    PacketAck(Box<PacketAck>),
    /// CoarseLocationUpdate packet
    CoarseLocationUpdate(Box<CoarseLocationUpdate>),
    /// CompleteaAgentMovementData packet
    CompleteAgentMovementData(Box<CompleteAgentMovementData>),
    /// AgentUpdate packet
    AgentUpdate(Box<AgentUpdate>),
    /// ChatFromSimulator packet
    ChatFromSimulator(Box<ChatFromSimulator>),
    /// ChatFromViewer packet
    ChatFromViewer(Box<ChatFromViewer>),
    /// StartPingCheck packet
    StartPingCheck(Box<StartPingCheck>),
    /// CompletePingCheckPacket
    CompletePingCheck(Box<CompletePingCheck>),
    /// RegionHandshakePacket
    RegionHandshake(Box<RegionHandshake>),
    /// RegionHandshakeReply packet
    RegionHandshakeReply(Box<RegionHandshakeReply>),
    /// LayerData packet
    LayerData(Box<LayerData>),
    /// AvatarAppearance packet
    AvatarAppearance(Box<AvatarAppearance>),
    /// ObjectUpdate packet
    ObjectUpdate(Box<ObjectUpdate>),
    /// AgentWearablesRequest packet
    AgentWearablesRequest(Box<AgentWearablesRequest>),
    /// AgentWearablesUpdate packet
    AgentWearablesUpdate(Box<AgentWearablesUpdate>),
    /// Send a request to the server for a logout
    LogoutRequest(Box<LogoutRequest>),

    // the following structs do not exist in the spec. These packets are only used to send data
    // back and forth from the UI.
    /// does not exist in spec. Used to send the decoded login resposne to the UI.
    LoginResponse(Box<LoginResponse>),
    /// Does not exist in spec. Used to send Login data  from the UI to the server.
    Login(Box<Login>),
    /// Does not exist in spec. Used to send errors to the UI.
    Error(Box<SessionError>),
    /// Does not exist in spec. Used to send Layer GTIF paths to the UI for rendering.
    MeshUpdate(Box<MeshUpdate>),
}

/// Functions for determining the type of the packet.
impl PacketType {
    /// if a packet is a UI event, assign it to the corresponding UI event struct.
    pub fn ui_event(&self) -> UiEventTypes {
        match self {
            PacketType::ChatFromSimulator(_) => UiEventTypes::ChatFromSimulatorEvent,
            PacketType::CoarseLocationUpdate(_) => UiEventTypes::CoarseLocationUpdateEvent,
            PacketType::DisableSimulator(_) => UiEventTypes::DisableSimulatorEvent,
            _ => UiEventTypes::None,
        }
    }
    /// map all of the packet types to their to_bytes functions.
    /// could probably be made into a macro.
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
            PacketType::StartPingCheck(data) => data.to_bytes(),
            PacketType::CompletePingCheck(data) => data.to_bytes(),
            PacketType::RegionHandshake(data) => data.to_bytes(),
            PacketType::RegionHandshakeReply(data) => data.to_bytes(),
            PacketType::LayerData(data) => data.to_bytes(),
            PacketType::AvatarAppearance(data) => data.to_bytes(),
            PacketType::ObjectUpdate(data) => data.to_bytes(),
            PacketType::AgentWearablesRequest(data) => data.to_bytes(),
            PacketType::LogoutRequest(data) => data.to_bytes(),
            PacketType::AgentWearablesUpdate(data) => data.to_bytes(),

            PacketType::LoginResponse(_) => Vec::new(),
            PacketType::Login(data) => data.to_bytes(),
            PacketType::Error(data) => data.to_bytes(),
            PacketType::MeshUpdate(data) => data.to_bytes(),
        }
    }
}

impl PacketType {
    /// Assigns a packet type based on frequency and ID from the header.
    /// This is how the spec handles deserialization, by sending the frequency and the header to
    /// define unique packets
    pub fn from_id(id: u16, frequency: PacketFrequency, bytes: &[u8]) -> io::Result<Self> {
        match frequency {
            PacketFrequency::High => match id {
                1 => Ok(PacketType::StartPingCheck(Box::new(
                    StartPingCheck::from_bytes(bytes)?,
                ))),
                2 => Ok(PacketType::CompletePingCheck(Box::new(
                    CompletePingCheck::from_bytes(bytes)?,
                ))),
                4 => Ok(PacketType::AgentUpdate(Box::new(AgentUpdate::from_bytes(
                    bytes,
                )?))),
                11 => Ok(PacketType::LayerData(Box::new(LayerData::from_bytes(
                    bytes,
                )?))),
                12 => Ok(PacketType::ObjectUpdate(Box::new(
                    ObjectUpdate::from_bytes(bytes)?,
                ))),
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
                80 => Ok(PacketType::ChatFromViewer(Box::new(
                    ChatFromViewer::from_bytes(bytes)?,
                ))),
                139 => Ok(PacketType::ChatFromSimulator(Box::new(
                    ChatFromSimulator::from_bytes(bytes)?,
                ))),
                148 => Ok(PacketType::RegionHandshake(Box::new(
                    RegionHandshake::from_bytes(bytes)?,
                ))),
                149 => Ok(PacketType::RegionHandshakeReply(Box::new(
                    RegionHandshakeReply::from_bytes(bytes)?,
                ))),
                152 => Ok(PacketType::DisableSimulator(Box::new(
                    DisableSimulator::from_bytes(bytes)?,
                ))),
                158 => Ok(PacketType::AvatarAppearance(Box::new(
                    AvatarAppearance::from_bytes(bytes)?,
                ))),
                249 => Ok(PacketType::CompleteAgentMovementData(Box::new(
                    CompleteAgentMovementData::from_bytes(bytes)?,
                ))),
                252 => Ok(PacketType::LogoutRequest(Box::new(
                    LogoutRequest::from_bytes(bytes)?,
                ))),
                382 => Ok(PacketType::AgentWearablesUpdate(Box::new(
                    AgentWearablesUpdate::from_bytes(bytes)?,
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

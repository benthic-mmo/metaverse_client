use super::{errors::SessionError, mesh_update::MeshUpdate};
use crate::{
    agent::coarse_location_update::CoarseLocationUpdate,
    chat::chat_from_simulator::ChatFromSimulator, core::disable_simulator::DisableSimulator,
    login::login_response::LoginResponse, packet::packet::PacketData,
    packet::packet_types::PacketType,
};
use core::fmt;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
/// Defines all of the packet types that should be handled as UI events and sent directly to the UI
/// on receive.
pub enum UiEventTypes {
    /// The LoginResponseEvent, containing the login response from the server
    LoginResponseEvent,
    /// Errors that the UI should display
    Error,
    /// Chats sent from the simulator
    ChatFromSimulatorEvent,
    /// Minimap update events
    CoarseLocationUpdateEvent,
    /// Viewer disconnects
    DisableSimulatorEvent,
    /// Render generated meshes
    MeshUpdate,

    /// for packets that are not UI events
    None,
}
impl UiEventTypes {
    /// Used for getting the UI event type from bytes
    pub fn packet_type_from_bytes(&self, data: &[u8]) -> Option<PacketType> {
        match self {
            UiEventTypes::LoginResponseEvent => {
                serde_json::from_str::<LoginResponse>(core::str::from_utf8(data).unwrap())
                    .ok()
                    .map(|packet| PacketType::LoginResponse(Box::new(packet)))
            }
            UiEventTypes::Error => {
                SessionError::from_bytes(data).map(|packet| PacketType::Error(Box::new(packet)))
            }
            UiEventTypes::MeshUpdate => {
                MeshUpdate::from_bytes(data).map(|packet| PacketType::MeshUpdate(Box::new(packet)))
            }
            UiEventTypes::ChatFromSimulatorEvent => ChatFromSimulator::from_bytes(data)
                .ok()
                .map(|packet| PacketType::ChatFromSimulator(Box::new(packet))),
            UiEventTypes::CoarseLocationUpdateEvent => CoarseLocationUpdate::from_bytes(data)
                .ok()
                .map(|packet| PacketType::CoarseLocationUpdate(Box::new(packet))),
            UiEventTypes::DisableSimulatorEvent => {
                Some(PacketType::DisableSimulator(Box::new(DisableSimulator {})))
            }
            _ => None, // Handle unimplemented cases
        }
    }
}

impl fmt::Display for UiEventTypes {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            UiEventTypes::LoginResponseEvent => write!(f, "LoginResponseEvent"),
            UiEventTypes::ChatFromSimulatorEvent => write!(f, "ChatFromSimulatorEvent"),
            UiEventTypes::CoarseLocationUpdateEvent => write!(f, "CoarseLocationUpdateEvent"),
            UiEventTypes::DisableSimulatorEvent => write!(f, "DisableSimulatorEvent"),
            UiEventTypes::MeshUpdate => write!(f, "LayerUpdateEvent"),
            UiEventTypes::None => write!(f, "None"),
            UiEventTypes::Error => write!(f, "Error"),
        }
    }
}

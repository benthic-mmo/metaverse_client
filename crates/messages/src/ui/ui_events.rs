use crate::{
    errors::SessionError, login_system::login_response::LoginResponse, packet::PacketData, packet_types::PacketType,
};
use core::fmt;

use serde::{Deserialize, Serialize};

use super::{chat_from_simulator::ChatFromSimulator, coarse_location_update::CoarseLocationUpdate, disable_simulator::DisableSimulator};


#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum UiEventTypes {
    LoginResponseEvent,
    Error,
    ChatFromSimulatorEvent,
    CoarseLocationUpdateEvent,
    DisableSimulatorEvent,
    LayerUpdateEvent,

    // for packets that are not events
    None,
}
impl UiEventTypes {
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
            UiEventTypes::LayerUpdateEvent => write!(f, "LayerUpdateEvent"),
            UiEventTypes::None => write!(f, "None"),
            UiEventTypes::Error => write!(f, "Error"),
        }
    }
}

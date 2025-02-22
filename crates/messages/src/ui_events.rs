use core::fmt;
use crate::{login_system::login_response::LoginResponse, packet::PacketData};

use serde::{Deserialize, Serialize};

use crate::{chat_from_simulator::ChatFromSimulator, coarse_location_update::CoarseLocationUpdate, packet_types::PacketType};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum UiEventTypes {
    LoginResponseEvent,
    ChatEvent,
    CoarseLocationUpdateEvent,
    DisableSimulatorEvent,
    // for packets that are not events
    None,
}
impl UiEventTypes{
pub fn packet_type_from_bytes(&self, data: &Vec<u8>) -> Option<PacketType> {
    match self {
        UiEventTypes::LoginResponseEvent => {
            serde_json::from_str::<LoginResponse>(&String::from_utf8(data.to_vec()).unwrap())
                .ok()
                .map(|packet| PacketType::LoginResponse(Box::new(packet)))
        }
        UiEventTypes::ChatEvent => {
            ChatFromSimulator::from_bytes(data)
                .ok()
                .map(|packet| PacketType::ChatFromSimulator(Box::new(packet)))
        }
        UiEventTypes::CoarseLocationUpdateEvent => {
            CoarseLocationUpdate::from_bytes(data)
                .ok()
                .map(|packet| PacketType::CoarseLocationUpdate(Box::new(packet)))
        }
        _ => None, // Handle unimplemented cases
    }
}
}

impl fmt::Display for UiEventTypes {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            UiEventTypes::LoginResponseEvent => write!(f, "LoginResponseEvent"),
            UiEventTypes::ChatEvent => write!(f, "ChatEvent"),
            UiEventTypes::CoarseLocationUpdateEvent => write!(f, "CoarseLocationUpdateEvent"),
            UiEventTypes::DisableSimulatorEvent => write!(f, "DisableSimulatorEvent"),
            UiEventTypes::None => write!(f, "None"),
        }
    }
}

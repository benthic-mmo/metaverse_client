use core::fmt;

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum UiEventTypes {
    LoginResponseEvent,
    ChatEvent,
    CoarseLocationUpdateEvent,
    DisableSimulatorEvent,
    // for packets that are not events
    None,
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

use crate::{
    errors::ParseError,
    udp::{
        agent::{agent_update::AgentUpdate, coarse_location_update::CoarseLocationUpdate},
        chat::chat_from_simulator::ChatFromSimulator,
        core::disable_simulator::DisableSimulator,
    },
    ui::{
        chat_from_viewer::ChatFromUI, errors::SessionError, land_update::LandUpdate,
        login_event::Login, login_response::LoginResponse, logout::Logout, mesh_update::MeshUpdate,
    },
};
use actix::Message;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Message, Clone)]
#[serde(tag = "type", content = "data")] // for json simplicity
#[rtype(result = "()")]
/// The type of the message used to notify the UI .
/// Some of these also implement PacketType, as they are part of the spec, but not all of them are.
pub enum UIResponse {
    /// Custom login event for sending from the UI to the core
    Login(Login),
    /// Message for sending chat events from the UI to the core to the server.
    ChatFromViewer(ChatFromUI),
    /// Message for sending logout events from the UI to the core to the server.
    Logout(Logout),

    /// Message for informing the server of updates to the agent, like head position, motion, and
    /// FOV.
    AgentUpdate(AgentUpdate),
}
impl UIResponse {
    /// converts a UiEvent object to bytes using serde.
    /// Bytes can be decoded directly as JSON objects
    pub fn to_bytes(&self) -> Vec<u8> {
        serde_json::to_vec(self).expect("Failed to serialize UIResponse")
    }

    /// Converts bytes to a UIEvent.
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, ParseError> {
        serde_json::from_slice(bytes).map_err(ParseError::SerdeError)
    }
}

#[derive(Serialize, Deserialize, Debug, Message, Clone)]
#[serde(tag = "type", content = "data")] // for json simplicity
#[rtype(result = "()")]
/// Message types for sending between the core and the UI. These are truncated types that do not
/// contain all of the values that their packet types contain.
pub enum UIMessage {
    /// Message for sending chat events from the server, to the core, to the UI
    ChatFromSimulator(ChatFromSimulator),
    /// Message for sending location update events from the server to the core to the UI.
    CoarseLocationUpdate(CoarseLocationUpdate),
    /// Custom LoginResponse event for notifying the UI of login
    LoginResponse(LoginResponse),
    /// Custom MeshUpdate event for notifying the UI of a mesh to render
    MeshUpdate(MeshUpdate),
    /// Custom SessionError event for notifying the UI of core errors
    Error(SessionError),
    /// Message for informing the core and UI of a server disconnect
    DisableSimulator(DisableSimulator),

    LandUpdate(LandUpdate),
}

impl UIMessage {
    /// converts a UiEvent object to bytes using serde.
    /// Bytes can be decoded directly as JSON objects
    pub fn to_bytes(&self) -> Vec<u8> {
        serde_json::to_vec(self).expect("Failed to serialize UIMessage")
    }

    /// Converts bytes to a UIEvent.
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, ParseError> {
        serde_json::from_slice(bytes).map_err(ParseError::SerdeError)
    }
}

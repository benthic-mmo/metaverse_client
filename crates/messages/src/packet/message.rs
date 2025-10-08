use crate::{
    agent::coarse_location_update::CoarseLocationUpdate,
    chat::{chat_from_simulator::ChatFromSimulator, chat_from_viewer::ChatFromViewer},
    core::disable_simulator::DisableSimulator,
    login::{login_response::LoginResponse, login_xmlrpc::Login},
    packet::errors::PacketError,
    ui::{errors::SessionError, mesh_update::MeshUpdate},
};
use actix::Message;
use bincode::{
    config,
    serde::{decode_from_slice, encode_to_vec},
};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Message)]
#[serde(tag = "type", content = "data")] // for json simplicity
#[rtype(result = "()")]
/// The type of the message used to notify the UI .
/// Some of these also implement PacketType, as they are part of the spec, but not all of them are.
pub enum EventType {
    /// Custom login event for sending from the UI to the core
    Login(Login),
    /// Custom LoginResponse event for notifying the UI of login
    LoginResponse(LoginResponse),
    /// Custom MeshUpdate event for notifying the UI of a mesh to render
    MeshUpdate(MeshUpdate),
    /// Custom SessionError event for notifying the UI of core errors
    Error(SessionError),
    /// Message for sending chat events from the server, to the core, to the UI
    ChatFromSimulator(ChatFromSimulator),
    /// Message for sending chat events from the UI to the core to the server.
    ChatFromViewer(ChatFromViewer),
    /// Message for sending location update events from the server to the core to the UI.
    CoarseLocationUpdate(CoarseLocationUpdate),
    /// Message for informing the core and UI of a server disconnect
    DisableSimulator(DisableSimulator),
}

impl EventType {
    /// converts a UiEvent object to bytes using serde.
    /// Bytes can be decoded directly as JSON objects
    pub fn to_bytes(&self) -> Vec<u8> {
        serde_json::to_vec(self).expect("Failed to serialize UiEvent")
    }

    /// Converts bytes to a UIEvent.
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, PacketError> {
        serde_json::from_slice(bytes).map_err(PacketError::SerdeDeserializeError)
    }
}

/// Format for sending a serialized message from the mailbox to the UI.
#[derive(Debug, Message, Serialize, Deserialize, Clone)]
#[rtype(result = "()")]
pub struct UiMessage {
    /// Which number in a series of messages is it
    pub sequence_number: u16,
    /// how mant messages are there in total
    pub total_packet_number: u16,
    /// for serializing
    pub packet_number: u16,
    /// the encoded message to be decoded by the UI
    pub message: Vec<u8>,
}

impl UiMessage {
    /// Convert the struct into bytes using JSON serialization
    /// This must be bincode, because the UIMessage allows for chunking. The internal structi s
    /// serialized with serde.
    pub fn to_bytes(&self) -> Vec<u8> {
        encode_to_vec(self, config::legacy()).expect("Failed to serialize UiMessage")
    }

    /// Convert bytes back into a `UiMessage` struct
    /// this must be bincode, because the UIMessgae allows for chunking. The internal struct is
    /// serialized with serde.
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, PacketError> {
        let (msg, _len): (Self, usize) = decode_from_slice(bytes, config::legacy())?;
        Ok(msg)
    }

    /// convert EventType to bytes and create a new UiMessage
    pub fn from_event(event: &EventType) -> Self {
        Self::new(event.to_bytes())
    }

    /// Convert UiMessage's message to an EventType
    pub fn into_event(self) -> Result<EventType, PacketError> {
        EventType::from_bytes(&self.message)
    }

    /// create a new UiMessage
    pub fn new(message: Vec<u8>) -> UiMessage {
        UiMessage {
            message,
            sequence_number: 0,
            total_packet_number: 0,
            packet_number: 0,
        }
    }
}

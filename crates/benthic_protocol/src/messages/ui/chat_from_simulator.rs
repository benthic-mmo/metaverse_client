use glam::Vec3;
use serde::{Deserialize, Serialize};

use uuid::Uuid;

use crate::messages::{
    ui::ui_messages::UIMessage,
    utils::chat_types::{Audible, ChatType, SourceType},
};

/// Implement UIMessage for ChatFromSimulator. Can be sent between the core and UI
impl UIMessage {
    pub fn new_chat_from_simulator(data: ChatFromSimulator) -> Self {
        UIMessage::ChatFromSimulator(data)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
/// Struct for notifying client of new chat messages
pub struct ChatFromSimulator {
    /// The person who sent the message
    pub from_name: String,
    /// the UUID of the source of the message
    pub source_id: Uuid,
    /// undocumented
    pub owner_id: Uuid,
    /// The type of agent that emitted the chat message
    pub source_type: SourceType,
    /// The type of chat, whisper, speak, shout, etc
    pub chat_type: ChatType,
    /// If the chat is audible to the user or not
    pub audible: Audible,
    /// position of the chat. Currently unused.
    pub position: Vec3,
    /// The contents of the message
    pub message: String,
}

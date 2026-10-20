use serde::{Deserialize, Serialize};

use crate::messages::{chat::chat_types::ChatType, ui_protocol::UIResponse};

#[derive(Serialize, Deserialize, Debug, Clone)]
/// the chat event sent from the UI
pub struct ChatFromUI {
    /// the message of the chat
    pub message: String,
    /// the type of the chat
    pub message_type: ChatType,
    /// the channel the message was sent on
    pub channel: i32,
}

impl UIResponse {
    /// Implement UiEvent for ChatFromViewer to allow it to be sent from the UI to the core
    pub fn new_chat_from_viewer(data: ChatFromUI) -> Self {
        UIResponse::ChatFromViewer(data)
    }
}

use serde::{Deserialize, Serialize};

use crate::{packet::message::UIResponse, udp::chat::ChatType};

#[derive(Serialize, Deserialize, Debug, Clone)]
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

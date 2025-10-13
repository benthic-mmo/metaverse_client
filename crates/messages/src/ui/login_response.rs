use crate::packet::message::UIMessage;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
/// The struct required for constructing a login
pub struct LoginResponse {
    pub firstname: String,
    pub lastname: String,
}

impl UIMessage {
    /// allow sending the login object between the UI and Core
    pub fn new_login_response_event(data: LoginResponse) -> Self {
        UIMessage::LoginResponse(data)
    }
}

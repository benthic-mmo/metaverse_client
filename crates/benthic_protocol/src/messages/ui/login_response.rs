use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::messages::ui::ui_messages::UIMessage;

#[derive(Debug, Clone, Serialize, Deserialize)]
/// The struct required for constructing a login
pub struct LoginResponse {
    /// first name of the logged in user
    pub firstname: String,
    /// last name of the logged in user
    pub lastname: String,
    /// agent ID of the logged in user
    pub agent_id: Uuid,
}

impl UIMessage {
    /// allow sending the login object between the UI and Core
    pub fn new_login_response_event(data: LoginResponse) -> Self {
        UIMessage::LoginResponse(data)
    }
}

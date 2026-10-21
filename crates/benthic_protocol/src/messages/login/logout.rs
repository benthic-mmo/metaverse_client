use crate::messages::ui_protocol::UIResponse;
use serde::{Deserialize, Serialize};

/// A message to request a logout from the core
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Logout {}

impl UIResponse {
    /// Implement UiEvent for ChatFromViewer to allow it to be sent from the UI to the core
    pub fn new_logout() -> Self {
        UIResponse::Logout(Logout {})
    }
}

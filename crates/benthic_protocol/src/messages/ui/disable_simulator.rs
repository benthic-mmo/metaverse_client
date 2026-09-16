use serde::{Deserialize, Serialize};

use crate::messages::ui::ui_messages::UIMessage;

impl UIMessage {
    /// create a new UI message to allow the client to inform the server of disconnects
    pub fn new_disable_simulator() -> Self {
        UIMessage::DisableSimulator(DisableSimulator {})
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
/// the disable simulator struct. Intentionally Contains no values.
pub struct DisableSimulator {}

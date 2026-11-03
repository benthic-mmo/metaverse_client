use crate::messages::ui_protocol::UIResponse;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrphanedObject {
    pub local_id: u32,
    pub parent_id: u32,
}

impl UIResponse {
    /// allow sending the login object between the UI and Core
    pub fn new_orphaned_object(data: OrphanedObject) -> Self {
        UIResponse::OrphanedObject(data)
    }
}

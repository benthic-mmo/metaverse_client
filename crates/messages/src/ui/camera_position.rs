use glam::Vec3;
use serde::{Deserialize, Serialize};

use crate::packet::message::UIMessage;

/// this is the struct for sending camera position updates from the core to the UI. These are sent
/// when the server wants to change the position of the camera manually
#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct CameraPosition {
    /// the position of the camera in the region
    pub position: Vec3,
}

impl UIMessage {
    /// creates a new CameraPosition message
    pub fn new_camera_position(data: CameraPosition) -> Self {
        UIMessage::CameraPosition(data)
    }
}

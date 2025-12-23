use glam::Vec3;
use serde::{Deserialize, Serialize};

use crate::packet::message::UIMessage;

/// this is the struct for sending mesh updates from the core to the UI.
/// the path is the path to the generated gltf file, and the position is where to place it in the
/// world.
#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct CameraPosition {
    /// the position of the camera in the region
    pub position: Vec3,
}

impl UIMessage {
    /// creates a new MeshUpdate message
    pub fn new_camera_position(data: CameraPosition) -> Self {
        UIMessage::CameraPosition(data)
    }
}

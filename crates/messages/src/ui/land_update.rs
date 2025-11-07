use std::path::PathBuf;

use glam::Vec3;
use serde::{Deserialize, Serialize};

use crate::packet::message::UIMessage;

/// this is the struct for sending mesh updates from the core to the UI.
/// the path is the path to the generated gltf file, and the position is where to place it in the
/// world.
#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct LandUpdate {
    pub path: PathBuf,
}

impl UIMessage {
    /// creates a new MeshUpdate message
    pub fn new_land_update(data: LandUpdate) -> Self {
        UIMessage::LandUpdate(data)
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct LandData {
    pub vertices: Vec<Vec3>,
    pub indices: Vec<u16>,
    pub position: Vec3,
}

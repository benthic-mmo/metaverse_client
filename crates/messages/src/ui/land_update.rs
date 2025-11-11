use std::path::PathBuf;

use glam::Vec3;
use serde::{Deserialize, Serialize};

use crate::packet::message::UIMessage;

/// Struct for sending land updates from the core to the UI.
/// Land is stored as a JSON array of serialized LandData, which can be used by the UI to generate
/// terrain which can be affected by shaders.
#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct LandUpdate {
    /// Path to the generated JSON
    pub path: PathBuf,
}

impl UIMessage {
    /// creates a new MeshUpdate message
    pub fn new_land_update(data: LandUpdate) -> Self {
        UIMessage::LandUpdate(data)
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
/// Struct for defining land updates used by the UI
pub struct LandData {
    /// list of terrain vertices
    pub vertices: Vec<Vec3>,
    /// indices of terrain vertices
    pub indices: Vec<u16>,
    /// position in-world where the terrain goes
    pub position: Vec3,
}

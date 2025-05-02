use glam::U16Vec2;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// this is the struct for sending LayerUpdates from the core to the UI.
/// the path is the path to the unpacke gltf file, and the position is where to place it in the
/// world.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LayerUpdate {
    pub path: PathBuf,
    pub position: U16Vec2,
}
impl LayerUpdate {
    pub fn to_bytes(&self) -> Vec<u8> {
        bincode::serialize(self).expect("Failed to serialize LayerUpdate")
    }
    pub fn from_bytes(bytes: &[u8]) -> Option<Self> {
        bincode::deserialize(bytes).ok()
    }
}

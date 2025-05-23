use glam::{U16Vec2, Vec3};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// this is the struct for sending LayerUpdates from the core to the UI.
/// the path is the path to the generated gltf file, and the position is where to place it in the
/// world.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MeshUpdate {
    /// path to the generated gtlf file that contains the layer's data
    pub path: PathBuf,
    /// position of the mesh to render
    pub position: Vec3,
}
impl MeshUpdate {
    /// convert the layer update to bytes to send to the UI
    pub fn to_bytes(&self) -> Vec<u8> {
        bincode::serialize(self).expect("Failed to serialize LayerUpdate")
    }
    /// convert the bytes back to a layer update struct
    pub fn from_bytes(bytes: &[u8]) -> Option<Self> {
        bincode::deserialize(bytes).ok()
    }
}

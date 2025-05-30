use glam::Vec3;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use uuid::Uuid;

/// this is the struct for sending mesh updates from the core to the UI.
/// the path is the path to the generated gltf file, and the position is where to place it in the
/// world.
#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct MeshUpdate {
    /// path to the generated gtlf file that contains the mesh
    pub path: PathBuf,
    /// Where to render the mesh
    pub position: Vec3,
    /// The type of mesh getting rendered. Land, Object, Avatar, etc.
    pub mesh_type: MeshType,
    /// ID of the mesh. For agents, this will be the AgentID.
    pub id: Option<Uuid>,
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
/// Type of mesh the UI is rendering.
pub enum MeshType {
    /// Land type
    Land,
    /// Avatar type
    #[default]
    Avatar,
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

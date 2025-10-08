use glam::Vec3;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use uuid::Uuid;

use crate::packet::message::EventType;

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

#[derive(Clone, Debug, Serialize, Deserialize, Default, PartialEq)]
/// Type of mesh the UI is rendering.
pub enum MeshType {
    /// Land type
    Land,
    /// Avatar type
    #[default]
    Avatar,
}

impl EventType {
    pub fn new_mesh_update(data: MeshUpdate) -> Self {
        EventType::MeshUpdate(data)
    }
}

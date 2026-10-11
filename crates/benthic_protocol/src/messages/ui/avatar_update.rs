use glam::{Quat, Vec3};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use uuid::Uuid;

use crate::messages::ui::ui_messages::UIMessage;

/// this is the struct for sendingavatar updates from the core to the UI.
/// the path is the path to the generated gltf file, and the position is where to place it in the
/// world.
#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct AvatarUpdate {
    /// path to the generated gtlf file that contains the avatar's gltf
    pub path: PathBuf,
    /// Where to render the mesh
    pub rotation: Quat,
    pub position: Vec3,
    /// ID of the mesh. For agents, this will be the AgentID.
    pub id: Option<Uuid>,
}

impl UIMessage {
    /// creates a new MeshUpdate message
    pub fn new_avatar_update(data: AvatarUpdate) -> Self {
        UIMessage::AvatarUpdate(data)
    }
}

use crate::packet::message::UIMessage;
use glam::Vec3;
use rgb::{Rgba, Zeroable};
use serde::{Deserialize, Serialize};

/// Struct for sending water updates from the core to the UI.
/// This is used for sending the height of the water, along with optional EEP values after load.
#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct SkyboxUpdate {
    pub sun_phase: f32,
}

impl UIMessage {
    /// creates a new MeshUpdate message
    pub fn new_skybox_update(data: SkyboxUpdate) -> Self {
        UIMessage::SkyboxUpdate(data)
    }
}

use crate::packet::message::UIMessage;
use rgb::Rgba;
use serde::{Deserialize, Serialize};

/// Struct for sending water updates from the core to the UI.
/// This is used for sending the height of the water, along with optional EEP values after load.
#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct WaterUpdate {
    pub height: f32,
    pub color: Rgba<f32>,
}

impl UIMessage {
    /// creates a new MeshUpdate message
    pub fn new_water_update(data: WaterUpdate) -> Self {
        UIMessage::WaterUpdate(data)
    }
}

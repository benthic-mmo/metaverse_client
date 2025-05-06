use std::collections::HashMap;

use glam::U16Vec2;
use metaverse_messages::ui::custom::layer_update::LayerUpdate;

use crate::layer_handler::PatchData;

/// TODO: unimplemented
#[derive(Debug, Clone)]
pub struct Wind;

impl PatchData for Wind {
    fn from_packet(
        _: &metaverse_messages::layer_data::LayerData,
        _: bool,
    ) -> Result<Vec<Self>, crate::error::PatchError> {
        Ok(vec![])
    }
    fn generate_ui_event(
        self: Self,
        _: &mut HashMap<U16Vec2, Self>,
        _: &HashMap<U16Vec2, Self>,
    ) -> Vec<LayerUpdate> {
        vec![]
    }
}

use crate::{error::PatchError, layer_handler::PatchData};
use glam::U16Vec2;
use metaverse_messages::{environment::layer_data::LayerData, ui::layer_update::LayerUpdate};
use std::collections::HashMap;

/// TODO: unimplemented
#[derive(Debug, Clone)]
pub struct Cloud;

impl PatchData for Cloud {
    fn from_packet(_: &LayerData, _: bool) -> Result<Vec<Self>, PatchError> {
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

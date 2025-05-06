use std::collections::HashMap;

use glam::U16Vec2;
use metaverse_messages::{layer_data::LayerData, ui::custom::layer_update::LayerUpdate};

use crate::{error::PatchError, layer_handler::PatchData};

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

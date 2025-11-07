use crate::{error::PatchError, layer_handler::PatchData};
use glam::U16Vec2;
use metaverse_messages::udp::environment::layer_data::LayerData;
use metaverse_messages::utils::render_data::RenderObject;
use std::collections::HashMap;

/// TODO: unimplemented
#[derive(Debug, Clone)]
pub struct Cloud;

impl PatchData for Cloud {
    fn from_packet(_: &LayerData, _: bool) -> Result<Vec<Self>, PatchError> {
        Ok(vec![])
    }
}

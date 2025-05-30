use crate::{error::PatchError, layer_handler::PatchData};
use glam::U16Vec2;
use metaverse_messages::capabilities::mesh::Mesh;
use metaverse_messages::environment::layer_data::LayerData;
use std::collections::HashMap;

/// TODO: unimplemented
#[derive(Debug, Clone)]
pub struct Water;

impl PatchData for Water {
    fn from_packet(_: &LayerData, _: bool) -> Result<Vec<Self>, PatchError> {
        Ok(vec![])
    }
    fn generate_mesh(
        self,
        _: &mut HashMap<U16Vec2, Self>,
        _: &HashMap<U16Vec2, Self>,
    ) -> Option<Mesh> {
        None
    }
}

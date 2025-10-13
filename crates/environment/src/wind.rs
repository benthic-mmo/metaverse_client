use crate::layer_handler::PatchData;
use glam::U16Vec2;
use metaverse_messages::http::mesh::Mesh;
use metaverse_messages::udp::environment::layer_data::LayerData;
use std::collections::HashMap;

/// TODO: unimplemented
#[derive(Debug, Clone)]
pub struct Wind;

impl PatchData for Wind {
    fn from_packet(_: &LayerData, _: bool) -> Result<Vec<Self>, crate::error::PatchError> {
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

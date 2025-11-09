use crate::{error::PatchError, layer_handler::PatchData};
use metaverse_messages::udp::environment::layer_data::LayerData;

/// TODO: unimplemented
#[derive(Debug, Clone)]
pub struct Water;

impl PatchData for Water {
    fn from_packet(_: &LayerData, _: bool) -> Result<Vec<Self>, PatchError> {
        Ok(vec![])
    }
}

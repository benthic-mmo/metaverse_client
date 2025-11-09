use crate::layer_handler::PatchData;
use metaverse_messages::udp::environment::layer_data::LayerData;

/// TODO: unimplemented
#[derive(Debug, Clone)]
pub struct Wind;

impl PatchData for Wind {
    fn from_packet(_: &LayerData, _: bool) -> Result<Vec<Self>, crate::error::PatchError> {
        Ok(vec![])
    }
}

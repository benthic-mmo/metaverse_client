use std::collections::HashMap;

use metaverse_messages::{
    capabilities::capabilities::Capability, core::object_update::ObjectUpdate,
};

pub async fn handle_object_update(
    _: ObjectUpdate,
    _: HashMap<Capability, String>,
) -> std::io::Result<String> {
    Ok("aaa".to_string())
}

use std::path::PathBuf;

use metaverse_messages::utils::object_types::ObjectType;
use uuid::Uuid;

#[derive(Debug)]
pub struct OutfitItem {
    pub name: String,
    pub item_id: Uuid,
    pub asset_id: Uuid,
    pub item_type: ObjectType,
    pub json_path: PathBuf,
    pub mesh_path: PathBuf,
}

pub mod cache;
pub mod errors;
pub mod initialize_sqlite;
pub mod inventory;

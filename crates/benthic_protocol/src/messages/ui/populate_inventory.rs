use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::messages::ui::ui_messages::UIMessage;

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct UiItem {
    pub name: String,
    pub item_id: Uuid,
    pub asset_id: Uuid,
    pub item_type: String,
    pub thumbnail: PathBuf,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum FolderContents {
    Folder(UiFolder),
    Item(UiItem),
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct UiFolder {
    pub name: String,
    pub id: Uuid,
    pub parent: Option<Uuid>,
    pub contents: Vec<FolderContents>,
}
#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct PopulateInventory {
    pub folder: UiFolder,
}

impl UIMessage {
    pub fn new_populate_inventory(data: PopulateInventory) -> Self {
        UIMessage::PopulateInventory(data)
    }
}

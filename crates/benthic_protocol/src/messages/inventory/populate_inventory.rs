use crate::messages::ui_protocol::UIMessage;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use uuid::Uuid;

/// Struct for item data sent to the UI
#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct UiItem {
    /// Name of the object
    pub name: String,
    /// ID of the object
    pub item_id: Uuid,
    /// Asset ID of the object
    pub asset_id: Uuid,
    /// Type of the object
    pub item_type: String,
    /// Path to object's thumbnail on disk
    pub thumbnail: PathBuf,
}
/// Struct for folder data to be sent to the UI
#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct UiFolder {
    /// Name of the folder
    pub name: String,
    /// UUID of the folder
    pub id: Uuid,
    /// Parent of the folder. Usually a folder.
    pub parent: Option<Uuid>,
    /// Contents of the folder
    pub contents: Vec<FolderContents>,
}

/// Enum for containing folder contents.
/// Folders can contain other folders, and UI items.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum FolderContents {
    /// object is a folder
    Folder(UiFolder),
    /// object is an item
    Item(UiItem),
}

/// Populate inventory strut to update the UI with a folder and its contents
#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct PopulateInventory {
    /// Folder data to send to the UI
    pub folder: UiFolder,
}

impl UIMessage {
    /// create a new populate inventory object to send to the UI
    pub fn new_populate_inventory(data: PopulateInventory) -> Self {
        UIMessage::PopulateInventory(data)
    }
}

use crate::{errors::InventoryError, initialize_sqlite::Inventory};
use benthic_protocol::messages::inventory::populate_inventory::{
    FolderContents, PopulateInventory, UiFolder, UiItem,
};
use metaverse_messages::utils::object_types::ObjectType;
use sqlx::{FromRow, Row, sqlite::SqliteRow};
use std::{path::PathBuf, str::FromStr};
use uuid::Uuid;

/// Represents an inventory folder for parsing and storing inventory data
#[derive(Debug, Clone)]
pub struct InventoryFolder {
    /// The ID of the folder
    pub id: Uuid,
    /// The owner ID of the folder
    pub owner_id: Uuid,
    /// The agent ID the folder belongs to
    pub agent_id: Uuid,
    /// The version of the folder, used for invalidating cache
    pub version: i32,
    /// Number of sub-folders inside the folder
    pub descendent_count: i32,
    /// When the folder was created
    pub created_at: chrono::NaiveDateTime,
    /// Bool to determine if the folder is fully downloaded
    pub fully_downloaded: bool,
    /// UUID of the parent folder, if it has one
    pub parent: Option<Uuid>,
}
impl<'r> FromRow<'r, SqliteRow> for InventoryFolder {
    fn from_row(row: &'r SqliteRow) -> Result<Self, sqlx::Error> {
        Ok(Self {
            id: Uuid::parse_str(row.try_get("id")?)
                .map_err(|e| sqlx::Error::Decode(Box::new(e)))?,

            owner_id: Uuid::parse_str(row.try_get("owner_id")?)
                .map_err(|e| sqlx::Error::Decode(Box::new(e)))?,

            agent_id: Uuid::parse_str(row.try_get("agent_id")?)
                .map_err(|e| sqlx::Error::Decode(Box::new(e)))?,

            version: row.try_get("version")?,
            descendent_count: row.try_get("descendent_count")?,
            created_at: row.try_get("created_at")?,
            fully_downloaded: row.try_get("fully_downloaded")?,
            parent: row
                .try_get("parent")
                .map_err(|e| sqlx::Error::Decode(Box::new(e)))?,
        })
    }
}

/// Inventory item struct.
/// this contains only the metadata of an inventory item. Real values need to be retrieved from the
/// server.
#[derive(Debug, Clone)]
pub struct InventoryItem {
    /// Name of the item
    pub name: String,
    /// unique ID of the instance of the item
    pub item_id: Uuid,
    /// asset ID for downloading full data from the server
    pub asset_id: Uuid,
    /// type of the item
    pub item_type: ObjectType,
    /// Thumbnail for the item for displaying in inventory
    /// TODO: this is unused as of right now but should be implemented
    pub thumbnail: PathBuf,
}
impl<'r> FromRow<'r, SqliteRow> for InventoryItem {
    fn from_row(row: &'r SqliteRow) -> Result<Self, sqlx::Error> {
        Ok(Self {
            name: row.try_get("name")?,
            asset_id: Uuid::parse_str(row.try_get("asset_id")?)
                .map_err(|e| sqlx::Error::Decode(Box::new(e)))?,
            item_id: Uuid::parse_str(row.try_get("item_id")?)
                .map_err(|e| sqlx::Error::Decode(Box::new(e)))?,
            item_type: ObjectType::from(row.try_get::<&str, _>("item_type")?),
            thumbnail: PathBuf::from_str(row.try_get("thumbnail")?)
                .map_err(|e| sqlx::Error::Decode(Box::new(e)))?,
        })
    }
}

impl Inventory {
    /// Handles fetching the full inventory from the server.
    ///
    /// This is used to generate UI messages to populate the inventory in the UI
    ///
    /// # Errors
    /// Returns an [`InventoryError`] on failure.
    ///
    /// # Side Effects
    /// [Storage: Filesystem]
    pub async fn fetch_inventory(&self) -> Result<Vec<PopulateInventory>, InventoryError> {
        let folders = sqlx::query_as::<_, InventoryFolder>(
            r#"
            SELECT *
            FROM folders
            "#,
        )
        .fetch_all(&self.db)
        .await?;
        let mut ui_folders = Vec::new();
        for folder in &folders {
            let category = sqlx::query(
                r#"
                SELECT name
                FROM categories
                WHERE id = ?
                "#,
            )
            .bind(folder.id.to_string())
            .fetch_optional(&self.db)
            .await?;

            let category_name: Option<String> =
                category.map(|row| row.try_get("name")).transpose()?;

            let items = sqlx::query_as::<_, InventoryItem>(
                r#"
                SELECT name, item_id, asset_id, item_type, thumbnail
                FROM items
                WHERE folder_id = ?
                "#,
            )
            .bind(folder.id.to_string())
            .fetch_all(&self.db)
            .await?;
            let mut ui_items = Vec::new();
            for item in items {
                ui_items.push(FolderContents::Item(UiItem {
                    name: item.name,
                    item_id: item.item_id,
                    item_type: item.item_type.to_string(),
                    asset_id: item.asset_id,
                    thumbnail: item.thumbnail,
                }))
            }

            ui_folders.push(PopulateInventory {
                folder: UiFolder {
                    name: category_name.unwrap_or_default(),
                    id: folder.id,
                    parent: folder.parent,
                    contents: ui_items,
                },
            });
        }

        Ok(ui_folders)
    }
}

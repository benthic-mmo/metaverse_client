use std::path::PathBuf;

use crate::{OutfitItem, errors::InventoryError, initialize_sqlite::Inventory};
use metaverse_messages::utils::object_types::ObjectType;
use sqlx::Row;
use uuid::Uuid;

impl Inventory {
    pub async fn get_current_outfit(&self) -> Result<Vec<OutfitItem>, InventoryError> {
        let folder_row = sqlx::query(
            r#"
        SELECT id
        FROM categories
        WHERE name = 'Current Outfit'
        LIMIT 1
        "#,
        )
        .fetch_one(&self.db)
        .await?;

        let folder_id: String = folder_row.get("id");

        let items_rows = sqlx::query(
            r#"
        SELECT name, item_id, asset_id, item_type, json, mesh
        FROM items
        WHERE folder_id = ?
        "#,
        )
        .bind(&folder_id)
        .fetch_all(&self.db)
        .await?;

        let mut result: Vec<OutfitItem> = Vec::new();

        for row in items_rows {
            let mut name: String = row.get("name");
            let mut item_id_str: Option<String> = row.get("item_id");
            let mut asset_id_str: Option<String> = row.get("asset_id");
            let mut item_type_str: Option<String> = row.get("item_type");
            let mut item_type = ObjectType::from(item_type_str.as_deref().unwrap_or_default());
            let mut json: Option<String> = row.get("json");
            let mut mesh: Option<String> = row.get("mesh");

            if item_type == ObjectType::Link
                && let Some(asset_id) = asset_id_str.as_deref()
                && let Ok(linked_row) = sqlx::query(
                    r#"
                    SELECT name, item_id, asset_id, item_type, json, mesh
                    FROM items
                    WHERE item_id = ?
                    "#,
                )
                .bind(asset_id)
                .fetch_one(&self.db)
                .await
            {
                name = linked_row.get("name");
                item_id_str = linked_row.get("item_id");
                asset_id_str = linked_row.get("asset_id");
                item_type_str = linked_row.get("item_type");
                item_type = ObjectType::from(item_type_str.as_deref().unwrap_or_default());
                json = linked_row.get("json");
                mesh = linked_row.get("mesh");
            }

            let item_id = match item_id_str {
                Some(v) => Uuid::parse_str(&v)?,
                None => continue,
            };

            let asset_id = match asset_id_str {
                Some(v) => Uuid::parse_str(&v)?,
                None => continue,
            };

            result.push(OutfitItem {
                name,
                item_id,
                asset_id,
                item_type,
                json_path: json.map(PathBuf::from).unwrap_or_default(),
                mesh_path: mesh.map(PathBuf::from).unwrap_or_default(),
            });
        }

        use std::collections::HashMap;

        let mut map: HashMap<Uuid, OutfitItem> = HashMap::new();
        for item in result {
            map.entry(item.item_id).or_insert(item);
        }

        Ok(map.into_values().collect())
    }
}

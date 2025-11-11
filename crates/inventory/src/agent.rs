use crate::errors::InventoryError;
use std::collections::HashSet;
use metaverse_messages::utils::object_types::ObjectType;
use sqlx::SqlitePool;
use uuid::Uuid;

use sqlx::Row; // needed for .get()

pub async fn get_agent_outfit(
    pool: &SqlitePool,
    agent_id: Uuid,
) -> Result<Vec<(String, Uuid, Uuid, ObjectType)>, InventoryError> {
    let folder_id = agent_id.to_string();

    let rows = sqlx::query(
        r#"
        SELECT name, item_id, asset_id, item_type
        FROM items
        WHERE folder_id = ?
        "#,
    )
    .bind(folder_id)
    .fetch_all(pool)
    .await?;

    let mut result = Vec::with_capacity(rows.len());

    for row in rows {
        // Use row.get::<Type, _>("column_name")
        let name: String = row.get("name");
        let item_id_str: Option<String> = row.get("item_id");
        let asset_id_str: Option<String> = row.get("asset_id");
        let item_type_str: Option<String> = row.get("item_type");

        let item_id = Uuid::parse_str(
            item_id_str
                .as_ref()
                .ok_or_else(|| InventoryError::Error("item_id is null".into()))?,
        )?;
        let asset_id = Uuid::parse_str(
            asset_id_str
                .as_ref()
                .ok_or_else(|| InventoryError::Error("asset_id is null".into()))?,
        )?;
        let item_type = ObjectType::from(item_type_str.as_deref().unwrap_or_default());

        result.push((name, item_id, asset_id, item_type));
    }

    Ok(result)
}

pub async fn get_current_outfit(
    pool: &SqlitePool,
) -> Result<HashSet<(Uuid, Uuid, ObjectType)>, InventoryError> {
    // Get the folder_id for "Current Outfit"
    let folder_row = sqlx::query(
        r#"
        SELECT id
        FROM categories
        WHERE name = 'Current Outfit'
        LIMIT 1
        "#,
    )
    .fetch_one(pool)
    .await?;

    let folder_id: String = folder_row.get("id");

    // Fetch items in that folder
    let items_rows = sqlx::query(
        r#"
        SELECT name, item_id, asset_id, item_type
        FROM items
        WHERE folder_id = ?
        "#,
    )
    .bind(&folder_id)
    .fetch_all(pool)
    .await?;

    let mut result: HashSet<(Uuid, Uuid, ObjectType)> = HashSet::new();

    for row in items_rows {
        let mut _name: String = row.get("name");
        let mut item_id_str: Option<String> = row.get("item_id");
        let mut asset_id_str: Option<String> = row.get("asset_id");
        let mut item_type_str: Option<String> = row.get("item_type");
        let mut item_type = ObjectType::from(item_type_str.as_deref().unwrap_or_default());

        // If the item is a link, fetch the actual linked item
        if item_type == ObjectType::Link
            && let Ok(linked_row) = sqlx::query(
                r#"
                SELECT name, item_id, asset_id, item_type
                FROM items
                WHERE item_id = ?
                "#,
            )
            .bind(asset_id_str.as_deref())
            .fetch_one(pool)
            .await
            {
                _name = linked_row.get("name");
                item_id_str = linked_row.get("item_id");
                asset_id_str = linked_row.get("asset_id");
                item_type_str = linked_row.get("item_type");
                item_type = ObjectType::from(item_type_str.as_deref().unwrap_or_default());
            }

        let item_id = Uuid::parse_str(item_id_str.as_deref().unwrap_or_default())?;
        let asset_id = Uuid::parse_str(asset_id_str.as_deref().unwrap_or_default())?;
result.insert((item_id, asset_id, item_type));
    }

    Ok(result)
}

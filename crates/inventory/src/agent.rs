use crate::errors::InventoryError;
use metaverse_agent::avatar::Avatar;
use metaverse_messages::utils::object_types::ObjectType;
use sqlx::SqlitePool;
use std::{
    path::PathBuf,
    time::{SystemTime, UNIX_EPOCH},
};
use uuid::Uuid;

use sqlx::Row; // needed for .get()
               //
pub struct OutfitItem {
    pub name: String,
    pub item_id: Uuid,
    pub asset_id: Uuid,
    pub item_type: ObjectType,
    pub json_path: PathBuf,
    pub mesh_path: PathBuf,
}

pub async fn sqlite_update_avatar(pool: &SqlitePool, avatar: Avatar) -> Result<(), InventoryError> {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64;
    let data = serde_json::to_string(&avatar)?;
    sqlx::query(
        r#"
        UPDATE agents
        SET last_update = ?,
            data = ?
        WHERE agent_id = ?
        "#,
    )
    .bind(now)
    .bind(data)
    .bind(avatar.agent_id.to_string())
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn sqlite_update_outfit_item_json_path(
    pool: &SqlitePool,
    asset_id: Uuid,
    json_path: &str,
) -> Result<(), InventoryError> {
    sqlx::query(
        r#"
        UPDATE items
        SET asset_id = ?,
            json = ?
        WHERE asset_id = ?
        "#,
    )
    .bind(asset_id.to_string())
    .bind(json_path)
    .bind(asset_id.to_string())
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn sqlite_update_outfit_item_mesh_path(
    pool: &SqlitePool,
    asset_id: Uuid,
    mesh_path: &str,
) -> Result<(), InventoryError> {
    sqlx::query(
        r#"
        UPDATE items
        SET mesh = ?
        WHERE asset_id = ?
        "#,
    )
    .bind(mesh_path)
    .bind(asset_id.to_string())
    .execute(pool)
    .await?;

    Ok(())
}

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

pub async fn sqlite_get_current_outfit_version(pool: &SqlitePool) -> Result<i32, InventoryError> {
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
    let version_row = sqlx::query(
        r#"
        SELECT version
        FROM folders
        WHERE id = ?
        "#,
    )
    .bind(&folder_id)
    .fetch_one(pool)
    .await?;

    let version: i32 = version_row.get("version");
    Ok(version)
}

pub async fn sqlite_get_current_avatar_version(
    pool: &SqlitePool,
    agent_id: String,
) -> Result<i32, InventoryError> {
    let agent_row = sqlx::query(
        r#"
        SELECT version
        FROM agents
        WHERE agent_id = ? 
        LIMIT 1
        "#,
    )
    .bind(&agent_id)
    .fetch_one(pool)
    .await?;
    let version: i32 = agent_row.get("version");
    Ok(version)
}

pub async fn sqlite_get_current_outfit(
    pool: &SqlitePool,
) -> Result<Vec<OutfitItem>, InventoryError> {
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

    let items_rows = sqlx::query(
        r#"
        SELECT name, item_id, asset_id, item_type, json, mesh
        FROM items
        WHERE folder_id = ?
        "#,
    )
    .bind(&folder_id)
    .fetch_all(pool)
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

        if item_type == ObjectType::Link {
            if let Some(asset_id) = asset_id_str.as_deref() {
                if let Ok(linked_row) = sqlx::query(
                    r#"
                    SELECT name, item_id, asset_id, item_type, json, mesh
                    FROM items
                    WHERE item_id = ?
                    "#,
                )
                .bind(asset_id)
                .fetch_one(pool)
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
            }
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

pub async fn sqlite_insert_avatar(
    pool: &SqlitePool,
    agent_id: Uuid,
    version: i32,
) -> Result<(), InventoryError> {
    sqlx::query(
        r#"
        INSERT INTO agents (
            agent_id,
            version
        )
        VALUES (?, ?)
        ON CONFLICT(agent_id) DO NOTHING
        "#,
    )
    .bind(agent_id.to_string())
    .bind(version)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn sqlite_get_avatar(
    pool: &SqlitePool,
    agent_id: Uuid,
) -> Result<Avatar, InventoryError> {
    let agent_row = sqlx::query(
        r#"
        SELECT data
        FROM agents
        WHERE agent_id = ?
        LIMIT 1
        "#,
    )
    .bind(agent_id.to_string())
    .fetch_one(pool)
    .await?;
    let data: String = agent_row.get("data");
    let avatar: Avatar = serde_json::from_str(&data)?;
    Ok(avatar)
}

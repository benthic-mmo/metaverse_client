use std::collections::HashSet;

use crate::errors::InventoryError;
use metaverse_messages::udp::object::object_update::ObjectUpdate;
use sqlx::{Row, SqlitePool};
use uuid::Uuid;

/// Store object update in DB
pub async fn insert_object_update(
    pool: &SqlitePool,
    update: &ObjectUpdate,
) -> Result<(), InventoryError> {
    let json = serde_json::to_string(update)?;
    sqlx::query(
        r#"
        INSERT INTO object_updates (id, full_id, parent, pcode, json)
        VALUES (?, ?, ?, ?, ?)
        ON CONFLICT(full_id) DO UPDATE SET
            id = excluded.id,
            parent = excluded.parent,
            pcode = excluded.pcode,
            json = excluded.json
        "#,
    )
    .bind(update.id as i64)
    .bind(update.full_id.to_string())
    .bind(update.pcode.to_string())
    .bind(update.parent_id.to_string())
    .bind(json)
    .execute(pool)
    .await?;
    Ok(())
}

/// Get object update by id
pub async fn get_object_update(
    pool: &SqlitePool,
    object_id: u32,
) -> Result<ObjectUpdate, InventoryError> {
    let row = sqlx::query("SELECT json FROM object_updates WHERE id = ?")
        .bind(object_id as i64)
        .fetch_one(pool)
        .await?;

    let json_str: String = row.try_get("json")?;
    let obj: ObjectUpdate = serde_json::from_str(&json_str)?;

    Ok(obj)
}

pub async fn get_missing_object_updates(
    pool: &SqlitePool,
    ids: &[Uuid],
) -> Result<Vec<Uuid>, sqlx::Error> {
    if ids.is_empty() {
        return Ok(vec![]);
    }

    let placeholders = vec!["?"; ids.len()].join(",");
    let query = format!(
        "SELECT item_id FROM object_updates WHERE full_id IN ({})",
        placeholders
    );

    let mut q = sqlx::query(&query);
    for id in ids {
        q = q.bind(id.to_string());
    }

    let rows = q.fetch_all(pool).await?;

    let existing_ids: HashSet<Uuid> = rows
        .iter()
        .filter_map(|row| row.try_get::<String, _>("full_id").ok())
        .filter_map(|s| Uuid::parse_str(&s).ok())
        .collect();

    let missing_ids: Vec<Uuid> = ids
        .iter()
        .copied()
        .filter(|id| !existing_ids.contains(id))
        .collect();

    Ok(missing_ids)
}

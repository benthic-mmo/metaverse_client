use std::collections::HashSet;

use crate::errors::InventoryError;
use glam::{Quat, Vec3};
use metaverse_messages::{
    udp::object::object_update::ObjectUpdate, utils::object_types::ObjectType,
};
use sqlx::{Row, SqlitePool};
use uuid::Uuid;

/// Store object update in DB
pub async fn insert_object_update(
    pool: &SqlitePool,
    update: &ObjectUpdate,
) -> Result<(), InventoryError> {
    let json = serde_json::to_string(update)?;

    let pos = update.motion_data.position;
    let rot = update.motion_data.rotation.normalize();
    let scale = update.scale;

    sqlx::query(
        r#"
        INSERT INTO object_updates (
            id, full_id, parent, pcode,
            pos_x, pos_y, pos_z,
            rot_x, rot_y, rot_z, rot_w,
            scale_x, scale_y, scale_z,
            json
        )
        VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
        ON CONFLICT(full_id) DO UPDATE SET
            id      = excluded.id,
            parent  = excluded.parent,
            pcode   = excluded.pcode,

            pos_x   = excluded.pos_x,
            pos_y   = excluded.pos_y,
            pos_z   = excluded.pos_z,

            rot_x   = excluded.rot_x,
            rot_y   = excluded.rot_y,
            rot_z   = excluded.rot_z,
            rot_w   = excluded.rot_w,

            scale_x = excluded.scale_x,
            scale_y = excluded.scale_y,
            scale_z = excluded.scale_z,

            json    = excluded.json
        "#,
    )
    .bind(update.id as i64)
    .bind(update.full_id.to_string())
    .bind(update.parent_id as i64)
    .bind(update.pcode.to_string())
    .bind(pos.x)
    .bind(pos.y)
    .bind(pos.z)
    .bind(rot.x)
    .bind(rot.y)
    .bind(rot.z)
    .bind(rot.w)
    .bind(scale.x)
    .bind(scale.y)
    .bind(scale.z)
    .bind(json)
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn insert_object_update_minimal(
    pool: &SqlitePool,
    id: u32,
    full_id: Uuid,
    object_type: ObjectType,
    parent_id: Option<u32>,
    position: Vec3,
    rotation: Quat,
    scale: Vec3,
) -> Result<(), InventoryError> {
    let parent_id = parent_id.unwrap_or(0);
    let rotation = rotation.normalize();

    sqlx::query(
        r#"
        INSERT INTO object_updates (
            id, full_id, parent, pcode,
            pos_x, pos_y, pos_z,
            rot_x, rot_y, rot_z, rot_w,
            scale_x, scale_y, scale_z
        )
        VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
        ON CONFLICT(full_id) DO UPDATE SET
            id      = excluded.id,
            parent  = excluded.parent,
            pcode   = excluded.pcode,

            pos_x   = excluded.pos_x,
            pos_y   = excluded.pos_y,
            pos_z   = excluded.pos_z,

            rot_x   = excluded.rot_x,
            rot_y   = excluded.rot_y,
            rot_z   = excluded.rot_z,
            rot_w   = excluded.rot_w,

            scale_x = excluded.scale_x,
            scale_y = excluded.scale_y,
            scale_z = excluded.scale_z
        "#,
    )
    .bind(id as i64)
    .bind(full_id.to_string())
    .bind(parent_id as i64)
    .bind(object_type.to_string())
    .bind(position.x)
    .bind(position.y)
    .bind(position.z)
    .bind(rotation.x)
    .bind(rotation.y)
    .bind(rotation.z)
    .bind(rotation.w)
    .bind(scale.x)
    .bind(scale.y)
    .bind(scale.z)
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

pub async fn get_object_scale_rotation_position(
    pool: &SqlitePool,
    object_id: u32,
) -> Result<(Vec3, Quat, Vec3), InventoryError> {
    let row = sqlx::query(
        r#"
        SELECT
            pos_x, pos_y, pos_z,
            rot_x, rot_y, rot_z, rot_w,
            scale_x, scale_y, scale_z
        FROM object_updates
        WHERE id = ?
        "#,
    )
    .bind(object_id as i64)
    .fetch_one(pool)
    .await?;

    let position = Vec3::new(
        row.try_get("pos_x")?,
        row.try_get("pos_y")?,
        row.try_get("pos_z")?,
    );

    let rotation = Quat::from_xyzw(
        row.try_get("rot_x")?,
        row.try_get("rot_y")?,
        row.try_get("rot_z")?,
        row.try_get("rot_w")?,
    )
    .normalize();

    let scale = Vec3::new(
        row.try_get("scale_x")?,
        row.try_get("scale_y")?,
        row.try_get("scale_z")?,
    );

    Ok((scale, rotation, position))
}

pub async fn set_object_transform_by_id(
    pool: &SqlitePool,
    object_id: u32,
    position: Vec3,
    rotation: Quat,
    scale: Vec3,
) -> Result<(), InventoryError> {
    // Always normalize quaternions coming from the wire
    let rotation = rotation.normalize();

    sqlx::query(
        r#"
        UPDATE object_updates
        SET
            pos_x   = ?,
            pos_y   = ?,
            pos_z   = ?,

            rot_x   = ?,
            rot_y   = ?,
            rot_z   = ?,
            rot_w   = ?,

            scale_x = ?,
            scale_y = ?,
            scale_z = ?
        WHERE id = ?
        "#,
    )
    // position
    .bind(position.x)
    .bind(position.y)
    .bind(position.z)
    // rotation (quat)
    .bind(rotation.x)
    .bind(rotation.y)
    .bind(rotation.z)
    .bind(rotation.w)
    // scale
    .bind(scale.x)
    .bind(scale.y)
    .bind(scale.z)
    // id
    .bind(object_id as i64)
    .execute(pool)
    .await?;

    Ok(())
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

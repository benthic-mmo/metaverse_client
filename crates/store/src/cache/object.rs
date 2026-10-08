use std::{collections::HashSet, path::PathBuf};

use crate::{errors::InventoryError, initialize_sqlite::ObjectCache};
use benthic_protocol::objects::{GeneratorObject, MeshObjectData};
use glam::{Quat, Vec3};
use metaverse_messages::{
    udp::object::object_update::ExtraParams,
    utils::{object_types::ObjectType, texture_entry::TextureEntry},
};
use sqlx::{Row, SqlitePool};
use uuid::Uuid;

fn vec3_from_row(
    row: &sqlx::sqlite::SqliteRow,
    x: &str,
    y: &str,
    z: &str,
) -> Result<Vec3, sqlx::Error> {
    Ok(Vec3::new(
        row.try_get::<f32, &str>(x)?,
        row.try_get::<f32, &str>(y)?,
        row.try_get::<f32, &str>(z)?,
    ))
}

fn quat_from_row(
    row: &sqlx::sqlite::SqliteRow,
    x: &str,
    y: &str,
    z: &str,
    w: &str,
) -> Result<Quat, sqlx::Error> {
    Ok(Quat::from_xyzw(
        row.try_get::<f32, &str>(x)?,
        row.try_get::<f32, &str>(y)?,
        row.try_get::<f32, &str>(z)?,
        row.try_get::<f32, &str>(w)?,
    ))
}

impl ObjectCache {
    pub async fn get_scale_rotation_position(
        &self,
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
        .fetch_one(&self.db)
        .await?;
        let position = vec3_from_row(&row, "pos_x", "pos_y", "pos_z")?;
        let scale = vec3_from_row(&row, "scale_x", "scale_y", "scale_z")?;
        let rotation = quat_from_row(&row, "rot_x", "rot_y", "rot_z", "rot_w")?;
        Ok((scale, rotation, position))
    }

    pub async fn update_glb_path(
        &self,
        full_id: Uuid,
        glb_path: &str,
    ) -> Result<(), InventoryError> {
        sqlx::query(
            r#"
        UPDATE object_updates
        SET glb = ?
        WHERE full_id = ?
        "#,
        )
        .bind(glb_path)
        .bind(full_id.to_string())
        .execute(&self.db)
        .await?;

        Ok(())
    }

    pub async fn update_json_path(
        &self,
        full_id: Uuid,
        asset_id: Uuid,
        json_path: &str,
    ) -> Result<(), InventoryError> {
        sqlx::query(
            r#"
        UPDATE object_updates
        SET asset_id = ?,
            json = ? 
        WHERE full_id = ?
        "#,
        )
        .bind(asset_id.to_string())
        .bind(json_path)
        .bind(full_id.to_string())
        .execute(&self.db)
        .await?;

        Ok(())
    }

    pub async fn check_cache(
        &self,
        id: u32,
        crc: u32,
        region_id: String,
    ) -> Result<(Uuid, PathBuf, Option<PathBuf>, GeneratorObject), InventoryError> {
        let row = sqlx::query(
            r#"
        SELECT
            asset_id,
            glb,
            full_id,
            id,
            parent,
            scale_x, scale_y, scale_z,
            rot_x, rot_y, rot_z, rot_w,
            pos_x, pos_y, pos_z
        FROM object_updates
        WHERE id = ?
          AND crc = ?
          AND region_id = ?
        "#,
        )
        .bind(id)
        .bind(crc)
        .bind(region_id)
        .fetch_optional(&self.db)
        .await?
        .ok_or_else(|| InventoryError::CacheMiss("Object not found in cache".to_string()))?;

        self.build_cache_result(row).await
    }
    pub async fn check_cache_by_asset_id(
        &self,
        asset_id: Uuid,
        full_id: Uuid,
    ) -> Result<(PathBuf, GeneratorObject), InventoryError> {
        let glb_row = sqlx::query(
            r#"
        SELECT glb
        FROM object_updates
        WHERE asset_id = ?
          AND glb IS NOT NULL
        "#,
        )
        .bind(asset_id)
        .fetch_optional(&self.db)
        .await?
        .ok_or_else(|| InventoryError::CacheMiss("GLB not found in cache".to_string()))?;

        let glb: String = glb_row.try_get("glb")?;
        let glb_path = PathBuf::from(glb);

        if !glb_path.exists() {
            return Err(InventoryError::CacheMiss(
                "GLB file not found on disk".to_string(),
            ));
        }

        let object_row = sqlx::query(
            r#"
        SELECT
            full_id,
            id,
            parent,
            scale_x, scale_y, scale_z,
            rot_x, rot_y, rot_z, rot_w,
            pos_x, pos_y, pos_z
        FROM object_updates
        WHERE full_id = ?
        "#,
        )
        .bind(full_id)
        .fetch_optional(&self.db)
        .await?
        .ok_or_else(|| InventoryError::CacheMiss("Object not found in cache".to_string()))?;

        let generator = GeneratorObject {
            full_id: {
                let id: String = object_row.try_get("full_id")?;
                Uuid::parse_str(&id)?
            },
            local_id: object_row.try_get("id")?,
            parent_id: object_row.try_get("parent")?,
            position: vec3_from_row(&object_row, "pos_x", "pos_y", "pos_z")?,
            scale: vec3_from_row(&object_row, "scale_x", "scale_y", "scale_z")?,
            rotation: quat_from_row(&object_row, "rot_x", "rot_y", "rot_z", "rot_w")?,
        };

        Ok((glb_path, generator))
    }

    async fn build_cache_result(
        &self,
        row: sqlx::sqlite::SqliteRow,
    ) -> Result<(Uuid, PathBuf, Option<PathBuf>, GeneratorObject), InventoryError> {
        let asset_id: String = row.try_get("asset_id")?;
        let asset_id = Uuid::parse_str(&asset_id)?;

        let glb: Option<String> = row.try_get("glb")?;
        let glb_path = glb.as_ref().map(PathBuf::from);

        let generator = GeneratorObject {
            full_id: {
                let id_str: String = row.try_get("full_id")?;
                Uuid::parse_str(&id_str)?
            },
            local_id: row.try_get("id")?,
            parent_id: row.try_get("parent")?,
            position: vec3_from_row(&row, "pos_x", "pos_y", "pos_z")?,
            scale: vec3_from_row(&row, "scale_x", "scale_y", "scale_z")?,
            rotation: quat_from_row(&row, "rot_x", "rot_y", "rot_z", "rot_w")?,
        };

        Ok((
            asset_id,
            glb_path.clone().unwrap_or_default(),
            glb_path,
            generator,
        ))
    }
    /// Get object update by id
    pub async fn get_parent(&self, object_id: u32) -> Result<Option<u32>, InventoryError> {
        let row = sqlx::query("SELECT parent FROM object_updates WHERE id = ?")
            .bind(object_id as i64)
            .fetch_one(&self.db)
            .await?;

        Ok(row.try_get("parent")?)
    }

    pub async fn update(
        &self,
        object: MeshObjectData<TextureEntry, ObjectType>,
    ) -> Result<(), InventoryError> {
        let parent_id = object.parent.unwrap_or(0);
        let rotation = object.rotation.normalize();
        sqlx::query(
            r#"
        INSERT INTO object_updates (
            id, full_id, crc, region_id, parent, pcode,
            pos_x, pos_y, pos_z,
            rot_x, rot_y, rot_z, rot_w,
            scale_x, scale_y, scale_z
        )
        VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
        ON CONFLICT(full_id) DO UPDATE SET
            id      = excluded.id,
            parent  = excluded.parent,
            crc     = excluded.crc,
            region_id = excluded.region_id,
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
        .bind(object.local_id as i64)
        .bind(object.full_id.to_string())
        .bind(object.crc)
        .bind(object.region_id)
        .bind(parent_id as i64)
        .bind(object.object_type.to_string())
        .bind(object.position.x)
        .bind(object.position.y)
        .bind(object.position.z)
        .bind(rotation.x)
        .bind(rotation.y)
        .bind(rotation.z)
        .bind(rotation.w)
        .bind(object.scale.x)
        .bind(object.scale.y)
        .bind(object.scale.z)
        .execute(&self.db)
        .await?;

        Ok(())
    }
}

async fn _set_object_transform_by_id(
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

async fn _get_missing_object_updates(
    pool: &SqlitePool,
    ids: &[Uuid],
) -> Result<Vec<Uuid>, sqlx::Error> {
    if ids.is_empty() {
        return Ok(Vec::new());
    }

    let mut query = sqlx::QueryBuilder::<sqlx::Sqlite>::new(
        "SELECT full_id FROM object_updates WHERE full_id IN (",
    );

    let mut separated = query.separated(", ");

    for id in ids {
        separated.push_bind(id.to_string());
    }

    separated.push_unseparated(")");

    let rows = query.build().fetch_all(pool).await?;

    let existing_ids: HashSet<Uuid> = rows
        .iter()
        .filter_map(|row| row.try_get::<String, _>("full_id").ok())
        .filter_map(|id| Uuid::parse_str(&id).ok())
        .collect();

    Ok(ids
        .iter()
        .copied()
        .filter(|id| !existing_ids.contains(id))
        .collect())
}

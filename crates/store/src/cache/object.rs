use crate::{errors::InventoryError, initialize_sqlite::CacheOperation};
use benthic_protocol::objects::types::ObjectInstanceInfo;
use glam::{Quat, Vec3};
use metaverse_messages::utils::object_types::ObjectType;
use sqlx::{Row, SqlitePool};
use std::{collections::VecDeque, path::PathBuf};
use tokio::sync::mpsc;
use uuid::Uuid;

/// Sqlite pool connection for object cache functions
#[derive(Clone, Debug)]
pub struct ObjectCache {
    /// the async pool of db connections
    pub db: SqlitePool,
    /// Queue of unprocessed cache operations
    pub queue: mpsc::Sender<CacheOperation>,
}

/// Defines cache operations for objects.
#[derive(Clone, Debug)]
pub enum QueuedObjectCacheOperation {
    /// Add a new instance of an object to the queue
    UpdateObject {
        /// Common information about an object's instance such as location, parent ID, full id, etc.
        object: ObjectInstanceInfo,
        /// crc for cache invalidation
        crc: u32,
        /// region where the object was updated
        region_id: String,
        /// type of the object
        object_type: ObjectType,
    },

    /// Update the transform of an object
    UpdateTransform {
        /// Full ID of the object to update the transform of
        full_id: Uuid,
        /// Position of the object
        position: Vec3,
        /// Rotation of the object
        rotation: Quat,
        /// Scale of the object
        scale: Vec3,
    },

    /// Update an object's render object json path
    /// this can be referenced if the glb fails to generate
    UpdateJsonPath {
        /// globally unique identifier of the object
        full_id: Uuid,
        /// asset ID of the mesh object
        asset_id: Uuid,
        /// path of the JSON file
        json_path: PathBuf,
    },

    /// Update an objects' glb path
    /// this ensures the cache knows about already-generated mesh files for renderable objects.
    UpdateGlbPath {
        /// Globally unique identifier of the object
        full_id: Uuid,
        /// path to generated glb
        glb_path: PathBuf,
    },
}

impl ObjectCache {
    /// flush the object's cache of all pending sqlite operations
    pub async fn flush_queue(
        db: &SqlitePool,
        batch: &mut VecDeque<QueuedObjectCacheOperation>,
    ) -> Result<Vec<u32>, InventoryError> {
        if batch.is_empty() {
            return Ok(Vec::new());
        }

        let mut tx = db.begin().await?;
        let mut updated_object_ids = Vec::new();

        for operation in batch.iter() {
            match operation {
                QueuedObjectCacheOperation::UpdateObject {
                    object,
                    crc,
                    region_id,
                    object_type,
                } => {
                    let parent_id = object.parent_id.unwrap_or(0);
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
                        id = excluded.id,
                        parent = excluded.parent,
                        crc = excluded.crc,
                        region_id = excluded.region_id,
                        pcode = excluded.pcode,
                        pos_x = excluded.pos_x,
                        pos_y = excluded.pos_y,
                        pos_z = excluded.pos_z,
                        rot_x = excluded.rot_x,
                        rot_y = excluded.rot_y,
                        rot_z = excluded.rot_z,
                        rot_w = excluded.rot_w,
                        scale_x = excluded.scale_x,
                        scale_y = excluded.scale_y,
                        scale_z = excluded.scale_z
                    "#,
                    )
                    .bind(object.local_id as i64)
                    .bind(object.full_id.to_string())
                    .bind(*crc)
                    .bind(region_id)
                    .bind(parent_id as i64)
                    .bind(object_type.to_string())
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
                    .execute(&mut *tx)
                    .await?;

                    updated_object_ids.push(object.local_id);
                }

                QueuedObjectCacheOperation::UpdateTransform {
                    full_id,
                    position,
                    rotation,
                    scale,
                } => {
                    let rotation = rotation.normalize();

                    sqlx::query(
                        r#"
                    UPDATE object_updates
                    SET pos_x = ?, pos_y = ?, pos_z = ?,
                        rot_x = ?, rot_y = ?, rot_z = ?, rot_w = ?,
                        scale_x = ?, scale_y = ?, scale_z = ?
                    WHERE full_id = ?
                    "#,
                    )
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
                    .bind(full_id.to_string())
                    .execute(&mut *tx)
                    .await?;
                }

                QueuedObjectCacheOperation::UpdateJsonPath {
                    full_id,
                    asset_id,
                    json_path,
                } => {
                    sqlx::query(
                        r#"
                    UPDATE object_updates
                    SET asset_id = ?, json = ?
                    WHERE full_id = ?
                    "#,
                    )
                    .bind(asset_id.to_string())
                    .bind(json_path.to_string_lossy().as_ref())
                    .bind(full_id.to_string())
                    .execute(&mut *tx)
                    .await?;
                }

                QueuedObjectCacheOperation::UpdateGlbPath { full_id, glb_path } => {
                    sqlx::query(
                        r#"
                    UPDATE object_updates
                    SET glb = ?
                    WHERE full_id = ?
                    "#,
                    )
                    .bind(glb_path.to_string_lossy().as_ref())
                    .bind(full_id.to_string())
                    .execute(&mut *tx)
                    .await?;
                }
            }
        }

        tx.commit().await?;

        // Clear only after a successful commit.
        batch.clear();

        Ok(updated_object_ids)
    }

    /// Retrieve scale, rotation and position for an object using its scene-local ID
    ///
    /// # Parameters
    /// - `object_id`: scene-local object ID
    ///
    /// # Errors
    /// Returns [`InventoryError`] if the object is not in the cache
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

    /// Updates object in cache with its generated GLB path
    ///
    /// # Parameters
    /// - `full_id`: The ID of the object to update
    /// - `glb_path`: The path on disk to the object's GLB
    ///
    /// # Errors
    /// Returns [`InventoryError`] if the object cannot be updated
    ///
    /// # Side Effects
    /// [Storage: SQLite]
    pub async fn update_glb_path(
        &self,
        full_id: Uuid,
        glb_path: &str,
    ) -> Result<(), InventoryError> {
        self.queue
            .send(CacheOperation::Object(
                QueuedObjectCacheOperation::UpdateGlbPath {
                    full_id,
                    glb_path: PathBuf::from(glb_path),
                },
            ))
            .await
            .map_err(|_| {
                InventoryError::CacheMiss("SQLite cache writer has stopped".to_string())
            })?;

        Ok(())
    }

    /// Updates object in cache with its generated JSON path
    ///
    /// # Parameters
    /// - `full_id`: The ID of the object to update
    /// - `asset_id`: The ID of the object's asset (Mesh UUID or something simliar)
    /// - `json_path`: The path on disk to the object's JSON
    ///
    /// # Errors
    /// Returns [`InventoryError`] if the update fails
    ///
    /// # Side Effects
    /// [Storage: SQLite]
    pub async fn update_json_path(
        &self,
        full_id: Uuid,
        asset_id: Uuid,
        json_path: &str,
    ) -> Result<(), InventoryError> {
        self.queue
            .send(CacheOperation::Object(
                QueuedObjectCacheOperation::UpdateJsonPath {
                    full_id,
                    asset_id,
                    json_path: PathBuf::from(json_path),
                },
            ))
            .await
            .map_err(|_| {
                InventoryError::CacheMiss("SQLite cache writer has stopped".to_string())
            })?;

        Ok(())
    }

    /// retrieve cache data for an object by its scene-local ID, CRC and region id.
    /// Returns UUID of the object, its GLB path, and an ObjectInstanceInfo object containing the
    /// information on where and what is being rendered
    ///
    /// # Parameters
    /// - `id`: u32 scene-local ID
    /// - `crc`: crc value to ensure cache is up to date
    /// - `region_id`: region where the object was downloaded (to prevent collisions)
    ///
    /// # Errors
    ///  Returns [`InventoryError`] if the object is not in cache
    pub async fn check_cache(
        &self,
        id: u32,
        crc: u32,
        region_id: String,
    ) -> Result<(Uuid, PathBuf, Option<PathBuf>, ObjectInstanceInfo), InventoryError> {
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

    /// Retrieve an object from the cache based on its asset_id
    /// Asset IDs are unique to the actual mesh data of the object. Full IDs and Scene-local IDs are unique to the
    /// object instance.
    ///
    /// # Parameters
    /// - `asset_id`: Asset ID of the object to retrieve
    /// - `full_id`: unique UUID of the object's instance
    ///
    /// # Errors
    /// Returns [`InventoryError`] if the object is not in cache
    pub async fn check_cache_by_asset_id(
        &self,
        asset_id: Uuid,
        full_id: Uuid,
    ) -> Result<(PathBuf, ObjectInstanceInfo), InventoryError> {
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

        let generator = ObjectInstanceInfo {
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

    // convert a cache result to objects
    // TODO: why is it returning the glb path twice
    async fn build_cache_result(
        &self,
        row: sqlx::sqlite::SqliteRow,
    ) -> Result<(Uuid, PathBuf, Option<PathBuf>, ObjectInstanceInfo), InventoryError> {
        let asset_id: String = row.try_get("asset_id")?;
        let asset_id = Uuid::parse_str(&asset_id)?;

        let glb: Option<String> = row.try_get("glb")?;
        let glb_path = glb.as_ref().map(PathBuf::from);

        let generator = ObjectInstanceInfo {
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

    /// Get object parent by ID
    ///
    /// # Parameters
    /// - `object_id`: scene-local u32 ID of the object
    ///
    /// # Errors
    /// Returns an [`InventoryError`] on failure
    pub async fn get_parent(&self, object_id: u32) -> Result<Option<u32>, InventoryError> {
        let row = sqlx::query("SELECT parent FROM object_updates WHERE id = ?")
            .bind(object_id as i64)
            .fetch_one(&self.db)
            .await?;

        Ok(row.try_get("parent")?)
    }

    /// Add the full object update to the cache
    ///
    /// This ensures ObjecUpdates don't get lost if they fail to get handled, and should allow for
    /// reprocessing.
    ///
    /// # Parameters
    /// - `object`: [`ObjectInstanceInfo`] object. Contains ID information and position
    /// - `crc`: crc of the object used elsewhere for cache validation
    /// - `region_id`: Region where the object was downloaded
    /// - `object_type`: Type of the object
    ///
    /// # Errors
    /// Returns an [`InventoryError`] on failure
    ///
    /// # Side Effects
    /// [Storage: SQLite]
    pub async fn update(
        &self,
        object: ObjectInstanceInfo,
        crc: u32,
        region_id: String,
        object_type: ObjectType,
    ) -> Result<(), InventoryError> {
        self.queue
            .send(CacheOperation::Object(
                QueuedObjectCacheOperation::UpdateObject {
                    object,
                    crc,
                    region_id,
                    object_type,
                },
            ))
            .await
            .map_err(|_| {
                InventoryError::CacheMiss("SQLite cache writer has stopped".to_string())
            })?;

        Ok(())
    }
}

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

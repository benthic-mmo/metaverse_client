use crate::errors::InventoryError;
use benthic_protocol::objects::types::ObjectInstanceInfo;
use glam::{Quat, Vec3};
use metaverse_messages::utils::object_types::ObjectType;
use serde::Deserialize;
use std::path::PathBuf;
use surrealdb::{
    Surreal,
    engine::local::Db,
    types::{SurrealValue, Value},
};
use uuid::Uuid;

pub(crate) const OBJECT_CACHE_SCHEMA: &str = r#"
DEFINE TABLE object_updates SCHEMAFULL;

DEFINE FIELD local_id    ON object_updates TYPE int;
DEFINE FIELD full_id     ON object_updates TYPE string;
DEFINE FIELD crc         ON object_updates TYPE int;
DEFINE FIELD region_id   ON object_updates TYPE string;
DEFINE FIELD parent      ON object_updates TYPE option<int>;
DEFINE FIELD pcode       ON object_updates TYPE option<string>;

DEFINE FIELD pos_x ON object_updates TYPE float;
DEFINE FIELD pos_y ON object_updates TYPE float;
DEFINE FIELD pos_z ON object_updates TYPE float;

DEFINE FIELD rot_x ON object_updates TYPE float;
DEFINE FIELD rot_y ON object_updates TYPE float;
DEFINE FIELD rot_z ON object_updates TYPE float;
DEFINE FIELD rot_w ON object_updates TYPE float;

DEFINE FIELD scale_x ON object_updates TYPE float;
DEFINE FIELD scale_y ON object_updates TYPE float;
DEFINE FIELD scale_z ON object_updates TYPE float;

DEFINE FIELD asset_id    ON object_updates TYPE option<string>;
DEFINE FIELD json        ON object_updates TYPE option<string>;
DEFINE FIELD glb         ON object_updates TYPE option<string>;
DEFINE FIELD update_path ON object_updates TYPE option<string>;

DEFINE INDEX object_updates_full_id ON object_updates FIELDS full_id UNIQUE CONCURRENTLY;
DEFINE INDEX object_updates_local_id ON object_updates FIELDS local_id CONCURRENTLY;
DEFINE INDEX object_updates_parent ON object_updates FIELDS parent CONCURRENTLY;
DEFINE INDEX object_updates_asset_id ON object_updates FIELDS asset_id CONCURRENTLY;
DEFINE INDEX object_updates_region_crc ON object_updates FIELDS region_id, crc CONCURRENTLY;
"#;

#[derive(Debug, Deserialize, SurrealValue)]
struct AvatarAgentRow {
    agent_id: String,
}

#[derive(Debug, Deserialize, SurrealValue)]
struct ObjectUpdateRow {
    full_id: String,
    local_id: u32,
    parent: Option<i64>,
    pos_x: f32,
    pos_y: f32,
    pos_z: f32,
    rot_x: f32,
    rot_y: f32,
    rot_z: f32,
    rot_w: f32,
    scale_x: f32,
    scale_y: f32,
    scale_z: f32,
    asset_id: Option<String>,
    glb: Option<String>,
    json: Option<String>,
    update_path: Option<String>,
}

#[derive(Debug, Deserialize, SurrealValue)]
struct GlbRow {
    glb: Option<String>,
}

#[derive(Debug, Deserialize, SurrealValue)]
struct UpdatePathRow {
    update_path: Option<String>,
}

#[derive(Debug, Deserialize, SurrealValue)]
struct ParentRow {
    parent: Option<i64>,
}

/// SurrealDB cache for object data.
#[derive(Clone, Debug)]
pub struct ObjectCache {
    pub db: Surreal<Db>,
}

impl ObjectCache {
    /// Retrieve scale, rotation and position using the scene-local ID.
    pub async fn get_scale_rotation_position(
        &self,
        local_id: u32,
    ) -> Result<(Vec3, Quat, Vec3), InventoryError> {
        let mut response = self
            .db
            .query("SELECT * FROM object_updates WHERE local_id = $local_id LIMIT 1")
            .bind(("local_id", local_id as i64))
            .await?;

        let mut rows: Vec<ObjectUpdateRow> = response.check()?.take(0)?;

        let row = rows
            .pop()
            .ok_or_else(|| InventoryError::CacheMiss("Object not found in cache".to_string()))?;

        let position = Vec3::new(row.pos_x, row.pos_y, row.pos_z);
        let rotation = Quat::from_xyzw(row.rot_x, row.rot_y, row.rot_z, row.rot_w);
        let scale = Vec3::new(row.scale_x, row.scale_y, row.scale_z);

        Ok((scale, rotation, position))
    }

    /// Update an object's generated GLB path.
    pub async fn update_glb_path(
        &self,
        full_id: Uuid,
        glb_path: &str,
    ) -> Result<(), InventoryError> {
        let mut response = self
            .db
            .query(
                r#"
                UPDATE object_updates
                SET glb = $glb_path
                WHERE full_id = $full_id;
                "#,
            )
            .bind(("full_id", full_id.to_string()))
            .bind(("glb_path", glb_path.to_owned()))
            .await?;

        response.check()?;

        Ok(())
    }

    /// Update an object's generated JSON path and asset ID.
    pub async fn update_json_path(
        &self,
        full_id: Uuid,
        asset_id: Uuid,
        json_path: &str,
    ) -> Result<(), InventoryError> {
        let mut response = self
            .db
            .query(
                r#"
                UPDATE object_updates
                SET
                    asset_id = $asset_id,
                    json = $json_path
                WHERE full_id = $full_id;
                "#,
            )
            .bind(("full_id", full_id.to_string()))
            .bind(("asset_id", asset_id.to_string()))
            .bind(("json_path", json_path.to_owned()))
            .await?;

        response.check()?;

        Ok(())
    }

    /// Retrieve an object by scene-local ID, CRC and region ID.
    pub async fn check_cache(
        &self,
        local_id: u32,
        crc: u32,
        region_id: String,
    ) -> Result<(Uuid, PathBuf, Option<PathBuf>, ObjectInstanceInfo), InventoryError> {
        let mut response = self
            .db
            .query(
                r#"
                SELECT * FROM object_updates
                WHERE local_id = $local_id
                  AND crc = $crc
                  AND region_id = $region_id
                LIMIT 1;
                "#,
            )
            .bind(("local_id", local_id as i64))
            .bind(("crc", crc as i64))
            .bind(("region_id", region_id))
            .await?;

        let mut rows: Vec<ObjectUpdateRow> = response.check()?.take(0)?;

        let row = rows
            .pop()
            .ok_or_else(|| InventoryError::CacheMiss("Object not found in cache".to_string()))?;

        self.build_cache_result(row)
    }

    /// Update an avatar's local ID mapping.
    pub async fn update_avatar(&self, local_id: u32, agent_id: Uuid) -> Result<(), InventoryError> {
        let mut response = self
            .db
            .query(
                r#"
                UPSERT type::record('agents', $agent_id)
                SET
                    agent_id = $agent_id,
                    local_id = $local_id,
                    last_update = time::now();
                "#,
            )
            .bind(("agent_id", agent_id.to_string()))
            .bind(("local_id", local_id as i64))
            .await?;

        response.check()?;

        Ok(())
    }

    /// Retrieve the full UUID using the scene-local ID.
    pub async fn get_full_id(&self, local_id: u32) -> Result<Uuid, InventoryError> {
        let row = fetch_object(
            &self.db,
            "SELECT * FROM object_updates WHERE local_id = $local_id LIMIT 1",
            "local_id",
            Value::Number((local_id as i64).into()),
        )
        .await?;

        Ok(Uuid::parse_str(&row.full_id)?)
    }

    /// Retrieve an avatar's agent UUID using its scene-local ID.
    pub async fn get_avatar_agent_id(&self, local_id: u32) -> Result<Uuid, InventoryError> {
        let mut response = self
            .db
            .query("SELECT agent_id FROM agents WHERE local_id = $local_id LIMIT 1")
            .bind(("local_id", local_id as i64))
            .await?;

        let mut rows: Vec<AvatarAgentRow> = response.check()?.take(0)?;

        let row = rows
            .pop()
            .ok_or_else(|| InventoryError::CacheMiss("Avatar agent not found".to_string()))?;

        Ok(Uuid::parse_str(&row.agent_id)?)
    }

    /// Retrieve the serialized update path using the scene-local ID.
    pub async fn get_update(&self, local_id: u32) -> Result<PathBuf, InventoryError> {
        let mut response = self
            .db
            .query("SELECT update_path FROM object_updates WHERE local_id = $local_id LIMIT 1")
            .bind(("local_id", local_id as i64))
            .await?;

        let mut rows: Vec<UpdatePathRow> = response.check()?.take(0)?;

        rows.pop()
            .and_then(|row| row.update_path)
            .map(PathBuf::from)
            .ok_or_else(|| InventoryError::CacheMiss("Update path not found".to_string()))
    }

    /// Retrieve an object using its asset ID and instance UUID.
    pub async fn check_cache_by_asset_id(
        &self,
        asset_id: Uuid,
        full_id: Uuid,
    ) -> Result<(PathBuf, ObjectInstanceInfo), InventoryError> {
        let mut response = self
            .db
            .query(
                r#"
                SELECT glb FROM object_updates
                WHERE asset_id = $asset_id
                  AND glb IS NOT NONE
                LIMIT 1;
                "#,
            )
            .bind(("asset_id", asset_id.to_string()))
            .await?;

        let mut rows: Vec<GlbRow> = response.check()?.take(0)?;

        let glb_path = rows
            .pop()
            .and_then(|row| row.glb)
            .map(PathBuf::from)
            .ok_or_else(|| InventoryError::CacheMiss("GLB not found in cache".to_string()))?;

        if !glb_path.exists() {
            return Err(InventoryError::CacheMiss(
                "GLB file not found on disk".to_string(),
            ));
        }

        let row = fetch_object(
            &self.db,
            "SELECT * FROM object_updates WHERE full_id = $full_id LIMIT 1",
            "full_id",
            Value::String(full_id.to_string().into()),
        )
        .await?;

        let generator = ObjectInstanceInfo {
            full_id: Uuid::parse_str(&row.full_id)?,
            local_id: row.local_id,
            parent_id: row.parent.map(|id| id as u32),
            position: Vec3::new(row.pos_x, row.pos_y, row.pos_z),
            scale: Vec3::new(row.scale_x, row.scale_y, row.scale_z),
            rotation: Quat::from_xyzw(row.rot_x, row.rot_y, row.rot_z, row.rot_w),
        };

        Ok((glb_path, generator))
    }

    /// Convert a cache result into object data.
    fn build_cache_result(
        &self,
        row: ObjectUpdateRow,
    ) -> Result<(Uuid, PathBuf, Option<PathBuf>, ObjectInstanceInfo), InventoryError> {
        let asset_id = Uuid::parse_str(
            row.asset_id
                .as_deref()
                .ok_or_else(|| InventoryError::CacheMiss("Asset ID not found".to_string()))?,
        )?;

        let glb_path = row.glb.map(PathBuf::from);

        let generator = ObjectInstanceInfo {
            full_id: Uuid::parse_str(&row.full_id)?,
            local_id: row.local_id,
            parent_id: row.parent.map(|id| id as u32),
            position: Vec3::new(row.pos_x, row.pos_y, row.pos_z),
            scale: Vec3::new(row.scale_x, row.scale_y, row.scale_z),
            rotation: Quat::from_xyzw(row.rot_x, row.rot_y, row.rot_z, row.rot_w),
        };

        Ok((
            asset_id,
            glb_path.clone().unwrap_or_default(),
            glb_path,
            generator,
        ))
    }

    /// Retrieve an object's parent using its scene-local ID.
    pub async fn get_parent(&self, local_id: u32) -> Result<Option<u32>, InventoryError> {
        let mut response = self
            .db
            .query("SELECT parent FROM object_updates WHERE local_id = $local_id LIMIT 1")
            .bind(("local_id", local_id as i64))
            .await?;

        let mut rows: Vec<ParentRow> = response.check()?.take(0)?;

        let row = rows
            .pop()
            .ok_or_else(|| InventoryError::CacheMiss("Object not found in cache".to_string()))?;

        Ok(row.parent.map(|id| id as u32))
    }

    /// Add or replace an object update in the cache.
    pub async fn update(
        &self,
        object: ObjectInstanceInfo,
        crc: u32,
        region_id: String,
        object_type: ObjectType,
        update_path: PathBuf,
    ) -> Result<(), InventoryError> {
        let parent_id = object.parent_id.map(|id| id as i64);
        let rotation = object.rotation.normalize();

        let mut response = self
            .db
            .query(
                r#"
                UPSERT object_updates
                SET
                    full_id = $full_id,
                    local_id = $local_id,
                    crc = $crc,
                    region_id = $region_id,
                    parent = $parent,
                    pcode = $pcode,

                    pos_x = $pos_x,
                    pos_y = $pos_y,
                    pos_z = $pos_z,

                    rot_x = $rot_x,
                    rot_y = $rot_y,
                    rot_z = $rot_z,
                    rot_w = $rot_w,

                    scale_x = $scale_x,
                    scale_y = $scale_y,
                    scale_z = $scale_z,

                    update_path = $update_path
                WHERE full_id = $full_id;
                "#,
            )
            .bind(("full_id", object.full_id.to_string()))
            .bind(("local_id", object.local_id as i64))
            .bind(("crc", crc as i64))
            .bind(("region_id", region_id))
            .bind(("parent", parent_id))
            .bind(("pcode", object_type.to_string()))
            .bind(("pos_x", object.position.x))
            .bind(("pos_y", object.position.y))
            .bind(("pos_z", object.position.z))
            .bind(("rot_x", rotation.x))
            .bind(("rot_y", rotation.y))
            .bind(("rot_z", rotation.z))
            .bind(("rot_w", rotation.w))
            .bind(("scale_x", object.scale.x))
            .bind(("scale_y", object.scale.y))
            .bind(("scale_z", object.scale.z))
            .bind(("update_path", update_path.to_string_lossy().into_owned()))
            .await?;

        response.check()?;

        Ok(())
    }
}

async fn fetch_object(
    db: &Surreal<Db>,
    query: &str,
    binding_name: &str,
    binding_value: Value,
) -> Result<ObjectUpdateRow, InventoryError> {
    let mut response = db.query(query).bind((binding_name, binding_value)).await?;

    let mut rows: Vec<ObjectUpdateRow> = response.check()?.take(0)?;

    rows.pop()
        .ok_or_else(|| InventoryError::CacheMiss("Object not found".to_string()))
}

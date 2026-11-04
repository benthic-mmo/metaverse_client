use crate::{
    OutfitItem,
    errors::{InventoryError, OutfitError},
    initialize_sqlite::Inventory,
};
use log::warn;
use metaverse_avatar::avatar::Avatar;
use serde::Deserialize;
use std::{
    path::PathBuf,
    time::{SystemTime, UNIX_EPOCH},
};
use surrealdb::{Surreal, engine::local::Db, types::SurrealValue};
use uuid::Uuid;

pub(crate) const AVATAR_CACHE_SCHEMA: &str = r#"  
-- AVATAR CACHE  
DEFINE TABLE agents SCHEMAFULL;  
  
DEFINE FIELD agent_id    ON agents TYPE string;  
DEFINE FIELD local_id    ON agents TYPE option<int>;  
DEFINE FIELD last_update ON agents TYPE option<int>;  
DEFINE FIELD version     ON agents TYPE option<int>;  
DEFINE FIELD path        ON agents TYPE option<string>;  
DEFINE FIELD data        ON agents TYPE option<string>;  
  
DEFINE INDEX agents_agent_id ON agents FIELDS agent_id UNIQUE;  
DEFINE INDEX agents_local_id ON agents FIELDS local_id;  
  "#;

/// SurrealDB cache for avatar and outfit data.
#[derive(Clone, Debug)]
pub struct AvatarCache {
    /// The embedded database connection.
    pub db: Surreal<Db>,
}

#[derive(Debug, Deserialize, SurrealValue)]
struct AvatarDataRow {
    data: Option<String>,
}

#[derive(Debug, Deserialize, SurrealValue)]
struct AvatarVersionRow {
    version: Option<i32>,
}

#[derive(Debug, Deserialize, SurrealValue)]
struct CategoryRow {
    id: String,
}

#[derive(Debug, Deserialize, SurrealValue)]
struct FolderVersionRow {
    version: i32,
}

impl AvatarCache {
    /// Update the cached avatar immediately.
    pub async fn update(&self, avatar: &Avatar, path: PathBuf) -> Result<(), InventoryError> {
        let updated_at = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as i64;

        let mut response = self
            .db
            .query(
                r#"
                UPDATE agents
                SET last_update = $updated_at,
                    path = $path
                WHERE agent_id = $agent_id;
                "#,
            )
            .bind(("agent_id", avatar.agent_id.to_string()))
            .bind(("updated_at", updated_at))
            .bind(("path", path.to_string_lossy().into_owned()))
            .await?;

        response.check()?;

        Ok(())
    }

    /// Update an outfit item's JSON path immediately.
    pub async fn update_outfit_item_json_path(
        &self,
        asset_id: Uuid,
        json_path: &str,
    ) -> Result<(), InventoryError> {
        let mut response = self
            .db
            .query(
                r#"
                UPDATE items
                SET json = $json_path
                WHERE asset_id = $asset_id;
                "#,
            )
            .bind(("asset_id", asset_id.to_string()))
            .bind(("json_path", json_path.to_owned()))
            .await?;

        response.check()?;

        Ok(())
    }

    /// Retrieve the objects in the user's current outfit.
    pub async fn current_outfit(
        &self,
        agent_id: Uuid,
        inventory: Inventory,
        current_outfit_id: Uuid,
    ) -> Result<(Option<Avatar>, Vec<OutfitItem>), OutfitError> {
        let outfit = inventory
            .get_current_outfit(current_outfit_id)
            .await
            .map_err(|error| OutfitError::InventoryRetrieve { error })?;

        let current_outfit_version = match get_current_outfit_version(&self.db).await {
            Ok(version) => version,
            Err(error) => {
                warn!("Failed to get current outfit version: {:?}", error);
                return Ok((None, outfit));
            }
        };

        // Read the cached version before ensuring the row exists.
        let avatar_current_version = get_current_avatar_version(&self.db, agent_id).await;

        // Ensure the avatar row exists immediately.
        ensure_avatar(&self.db, agent_id, current_outfit_version)
            .await
            .map_err(|error| OutfitError::AvatarCacheFailure { error })?;

        let avatar = match avatar_current_version {
            Ok(Some(version)) if version == current_outfit_version => {
                match get_avatar(&self.db, agent_id).await {
                    Ok(avatar) => Some(avatar),
                    Err(error) => {
                        warn!("Avatar cache invalid; rebuilding: {:?}", error);
                        None
                    }
                }
            }
            Ok(_) | Err(_) => None,
        };

        Ok((avatar, outfit))
    }
}

/// Ensure an avatar cache row exists.
async fn ensure_avatar(
    db: &Surreal<Db>,
    agent_id: Uuid,
    version: i32,
) -> Result<(), InventoryError> {
    let mut response = db
        .query(
            r#"  
            UPSERT type::record('agents', $agent_id)  
            MERGE {  
                agent_id: $agent_id,  
                version: $version  
            };  
            "#,
        )
        .bind(("agent_id", agent_id.to_string()))
        .bind(("version", version))
        .await?;

    response.check()?;

    Ok(())
}

/// Retrieve a cached avatar.
async fn get_avatar(db: &Surreal<Db>, agent_id: Uuid) -> Result<Avatar, InventoryError> {
    let mut response = db
        .query("SELECT data FROM agents WHERE agent_id = $agent_id LIMIT 1")
        .bind(("agent_id", agent_id.to_string()))
        .await?;

    let mut rows: Vec<AvatarDataRow> = response.check()?.take(0)?;

    let data = rows
        .pop()
        .and_then(|row| row.data)
        .ok_or_else(|| InventoryError::CacheMiss("Avatar data not found".to_string()))?;

    Ok(serde_json::from_str(&data)?)
}

/// Retrieve the current outfit folder's version.
async fn get_current_outfit_version(db: &Surreal<Db>) -> Result<i32, InventoryError> {
    let mut response = db
        .query("SELECT id FROM categories WHERE name = 'Current Outfit' LIMIT 1")
        .await?;

    let mut rows: Vec<CategoryRow> = response.check()?.take(0)?;

    let category = rows.pop().ok_or_else(|| {
        InventoryError::CacheMiss("Current Outfit category not found".to_string())
    })?;

    let mut response = db
        .query("SELECT version FROM folders WHERE id = $id LIMIT 1")
        .bind(("id", category.id))
        .await?;

    let mut rows: Vec<FolderVersionRow> = response.check()?.take(0)?;

    let folder = rows
        .pop()
        .ok_or_else(|| InventoryError::CacheMiss("Current Outfit folder not found".to_string()))?;

    Ok(folder.version)
}

/// Retrieve the cached avatar's outfit version.
async fn get_current_avatar_version(
    db: &Surreal<Db>,
    agent_id: Uuid,
) -> Result<Option<i32>, InventoryError> {
    let mut response = db
        .query("SELECT version FROM agents WHERE agent_id = $agent_id LIMIT 1")
        .bind(("agent_id", agent_id.to_string()))
        .await?;

    let mut rows: Vec<AvatarVersionRow> = response.check()?.take(0)?;

    Ok(rows.pop().and_then(|row| row.version))
}

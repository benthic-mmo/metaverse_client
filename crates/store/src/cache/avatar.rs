use crate::{
    OutfitItem,
    errors::{InventoryError, OutfitError},
    initialize_sqlite::{AvatarCache, Inventory},
};
use log::warn;
use metaverse_avatar::avatar::Avatar;
use sqlx::{Row, SqlitePool};
use std::time::{SystemTime, UNIX_EPOCH};
use uuid::Uuid;

impl AvatarCache {
    /// Updates avatar cache with the serde json of the avatar
    ///
    /// # Parameters
    /// - `avatar`: The avatar to add to the cache
    ///
    /// # Side Effects
    /// [Storage: SQLite]
    pub async fn update(&self, avatar: Avatar) -> Result<(), InventoryError> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;
        let data = serde_json::to_vec(&avatar)?;
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
        .execute(&self.db)
        .await?;

        Ok(())
    }

    /// Updates avatar cache with outfit item's JSON path
    ///
    /// # Parameters
    /// - `asset_id`: The UUID of the object to update
    /// - `json_path`: The path of the JSON to pupdate
    ///
    /// # Side Effects
    /// [Storage: SQLite]
    pub async fn update_outfit_item_json_path(
        &self,
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
        .execute(&self.db)
        .await?;

        Ok(())
    }

    /// Retrieve all of the objects in the user avatar's current outfit
    ///
    /// # Parameters
    /// - `agent_id`: The UUID of the agent to retrieve
    /// - `inventory`: The inventory object (this also contains a db connection )
    /// - `current_outfit_id`: The uuid of the current outfit folder in the inventory
    pub async fn current_outfit(
        &self,
        agent_id: Uuid,
        inventory: Inventory,
        current_outfit_id: Uuid,
    ) -> Result<(Option<Avatar>, Vec<OutfitItem>), OutfitError> {
        let outfit = inventory
            .get_current_outfit(current_outfit_id)
            .await
            .map_err(|e| OutfitError::InventoryRetrieve { error: e })?;

        let current_outfit_version = match sqlite_get_current_outfit_version(&self.db).await {
            Ok(version) => version,
            Err(e) => {
                warn!("Failed to get current outfit version: {:?}", e);
                return Ok((None, outfit));
            }
        };

        let avatar_current_version =
            sqlite_get_current_avatar_version(&self.db, agent_id.to_string()).await;

        sqlite_insert_avatar(&self.db, agent_id, current_outfit_version)
            .await
            .map_err(|e| OutfitError::AvatarCacheFailure { error: e })?;

        let avatar = match avatar_current_version {
            Ok(version) if version == current_outfit_version => {
                match sqlite_get_avatar(&self.db, agent_id).await {
                    Ok(avatar) => Some(avatar),
                    Err(e) => {
                        warn!("Avatar cache invalid; rebuilding: {:?}", e);
                        None
                    }
                }
            }
            Ok(_) => None,
            Err(_) => None,
        };

        Ok((avatar, outfit))
    }
}

async fn sqlite_insert_avatar(
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

async fn sqlite_get_avatar(pool: &SqlitePool, agent_id: Uuid) -> Result<Avatar, InventoryError> {
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

    let data: Vec<u8> = agent_row.get("data");
    let avatar: Avatar = serde_json::from_slice(&data)?;

    Ok(avatar)
}

async fn sqlite_get_current_outfit_version(pool: &SqlitePool) -> Result<i32, InventoryError> {
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

async fn sqlite_get_current_avatar_version(
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

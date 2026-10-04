use crate::{
    OutfitItem,
    errors::{InventoryError, OutfitError},
    initialize_sqlite::{AvatarCache, Inventory},
};
use benthic_protocol::session::cache_enabled;
use log::warn;
use metaverse_avatar::avatar::Avatar;
use sqlx::{Row, SqlitePool};
use std::time::{SystemTime, UNIX_EPOCH};
use uuid::Uuid;

impl AvatarCache {
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

    pub async fn current_outfit(
        &self,
        agent_id: Uuid,
        inventory: Inventory,
    ) -> Result<(Option<Avatar>, Vec<OutfitItem>), OutfitError> {
        let outfit = inventory
            .get_current_outfit()
            .await
            .map_err(|e| OutfitError::InventoryRetrieve { error: e })?;

        let avatar = if cache_enabled() {
            let current_outfit_version = match sqlite_get_current_outfit_version(&self.db).await {
                Ok(version) => version,
                Err(e) => {
                    warn!("Failed to get current outfit version {:?}", e);
                    0
                }
            };

            let avatar_current_version =
                match sqlite_get_current_avatar_version(&self.db, agent_id.to_string()).await {
                    Ok(version) => version,
                    Err(e) => {
                        warn!("Failed to get current avatar version {:?}", e);
                        0
                    }
                };

            sqlite_insert_avatar(&self.db, agent_id, current_outfit_version)
                .await
                .map_err(|e| OutfitError::AvatarCacheFailure { error: e })?;

            if current_outfit_version == avatar_current_version {
                Some(
                    sqlite_get_avatar(&self.db, agent_id)
                        .await
                        .map_err(|e| OutfitError::AvatarDataRetrieveFailure { error: e })?,
                )
            } else {
                None
            }
        } else {
            None
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

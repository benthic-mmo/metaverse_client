use crate::{
    OutfitItem,
    errors::{InventoryError, OutfitError},
    initialize_sqlite::{CacheOperation, Inventory},
};
use log::warn;
use metaverse_avatar::avatar::Avatar;
use sqlx::{Row, SqlitePool};
use std::{
    collections::VecDeque,
    time::{SystemTime, UNIX_EPOCH},
};
use tokio::sync::mpsc;
use uuid::Uuid;

/// SQLite cache for avatar and outfit data.
#[derive(Clone, Debug)]
pub struct AvatarCache {
    /// The async pool of database connections.
    pub db: SqlitePool,
    /// Shared queue for cache write operations.
    pub queue: mpsc::Sender<CacheOperation>,
}

/// A queued write operation for the avatar cache.
#[derive(Debug)]
pub enum QueuedAvatarCacheOperation {
    /// Update the cached avatar data.
    Update {
        /// The agent ID to update
        agent_id: Uuid,
        /// The current time of the update
        updated_at: i64,
        /// The byte data to populate the update with
        data: Vec<u8>,
    },
    /// Ensure an avatar cache row exists.
    Ensure {
        /// The agent ID to ensure exists
        agent_id: Uuid,
        /// The version of the avatar to update
        version: i32,
    },
    /// Update the JSON path for an outfit item.
    UpdateOutfitItemJsonPath {
        /// Asset ID of the item
        asset_id: Uuid,
        /// Path to the JSON on disk where the outfit's renderable mesh is
        json_path: String,
    },
}

impl AvatarCache {
    /// Queue an update to the cached avatar.
    pub async fn update(&self, avatar: Avatar) -> Result<(), InventoryError> {
        let updated_at = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as i64;

        let agent_id = avatar.agent_id;
        let data = serde_json::to_vec(&avatar)?;

        self.queue
            .send(CacheOperation::Avatar(QueuedAvatarCacheOperation::Update {
                agent_id,
                updated_at,
                data,
            }))
            .await
            .map_err(|_| InventoryError::CacheQueueClosed)?;

        Ok(())
    }

    /// Queue an update to an outfit item's JSON path.
    pub async fn update_outfit_item_json_path(
        &self,
        asset_id: Uuid,
        json_path: &str,
    ) -> Result<(), InventoryError> {
        self.queue
            .send(CacheOperation::Avatar(
                QueuedAvatarCacheOperation::UpdateOutfitItemJsonPath {
                    asset_id,
                    json_path: json_path.to_owned(),
                },
            ))
            .await
            .map_err(|_| InventoryError::CacheQueueClosed)?;

        Ok(())
    }

    /// Apply a batch of avatar cache operations in one transaction.
    ///
    /// Operations remain in `batch` if the transaction fails.
    /// The batch is cleared only after a successful commit.
    pub async fn flush_queue(
        pool: &SqlitePool,
        batch: &mut VecDeque<QueuedAvatarCacheOperation>,
    ) -> Result<usize, InventoryError> {
        if batch.is_empty() {
            return Ok(0);
        }

        let mut tx = pool.begin().await?;

        for operation in batch.iter() {
            match operation {
                QueuedAvatarCacheOperation::Update {
                    agent_id,
                    updated_at,
                    data,
                } => {
                    sqlx::query(
                        r#"
                        UPDATE agents
                        SET last_update = ?,
                            data = ?
                        WHERE agent_id = ?
                        "#,
                    )
                    .bind(updated_at)
                    .bind(data)
                    .bind(agent_id.to_string())
                    .execute(&mut *tx)
                    .await?;
                }

                QueuedAvatarCacheOperation::Ensure { agent_id, version } => {
                    sqlx::query(
                        r#"
                        INSERT INTO agents (agent_id, version)
                        VALUES (?, ?)
                        ON CONFLICT(agent_id) DO NOTHING
                        "#,
                    )
                    .bind(agent_id.to_string())
                    .bind(version)
                    .execute(&mut *tx)
                    .await?;
                }

                QueuedAvatarCacheOperation::UpdateOutfitItemJsonPath {
                    asset_id,
                    json_path,
                } => {
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
                    .execute(&mut *tx)
                    .await?;
                }
            }
        }

        let operation_count = batch.len();

        tx.commit().await?;
        batch.clear();

        Ok(operation_count)
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

        let current_outfit_version = match sqlite_get_current_outfit_version(&self.db).await {
            Ok(version) => version,
            Err(error) => {
                warn!("Failed to get current outfit version: {:?}", error);
                return Ok((None, outfit));
            }
        };

        // Read the cached version before ensuring the row exists.
        let avatar_current_version = sqlite_get_current_avatar_version(&self.db, agent_id).await;

        // Preserve the existing behavior: ensure the row exists immediately.
        sqlite_insert_avatar(&self.db, agent_id, current_outfit_version)
            .await
            .map_err(|error| OutfitError::AvatarCacheFailure { error })?;

        let avatar = match avatar_current_version {
            Ok(version) if version == current_outfit_version => {
                match sqlite_get_avatar(&self.db, agent_id).await {
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
async fn sqlite_insert_avatar(
    pool: &SqlitePool,
    agent_id: Uuid,
    version: i32,
) -> Result<(), InventoryError> {
    sqlx::query(
        r#"
        INSERT INTO agents (agent_id, version)
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

/// Retrieve a cached avatar.
async fn sqlite_get_avatar(pool: &SqlitePool, agent_id: Uuid) -> Result<Avatar, InventoryError> {
    let row = sqlx::query(
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

    let data: Vec<u8> = row.get("data");
    let avatar = serde_json::from_slice(&data)?;

    Ok(avatar)
}

/// Retrieve the current outfit folder's version.
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

    Ok(version_row.get("version"))
}

/// Retrieve the cached avatar's outfit version.
async fn sqlite_get_current_avatar_version(
    pool: &SqlitePool,
    agent_id: Uuid,
) -> Result<i32, InventoryError> {
    let row = sqlx::query(
        r#"
        SELECT version
        FROM agents
        WHERE agent_id = ?
        LIMIT 1
        "#,
    )
    .bind(agent_id.to_string())
    .fetch_one(pool)
    .await?;

    Ok(row.get("version"))
}

use log::warn;
use metaverse_avatar::avatar::Avatar;
use sqlx::{Pool, Sqlite};
use uuid::Uuid;

use crate::{
    agent::{
        OutfitItem, sqlite_get_avatar, sqlite_get_current_avatar_version,
        sqlite_get_current_outfit, sqlite_get_current_outfit_version, sqlite_insert_avatar,
    },
    errors::OutfitError,
};

pub async fn cache_current_outfit(
    db_conn: Pool<Sqlite>,
    agent_id: Uuid,
) -> Result<(Option<Avatar>, Vec<OutfitItem>), OutfitError> {
    let outfit = sqlite_get_current_outfit(&db_conn)
        .await
        .map_err(|e| OutfitError::InventoryRetrieve { error: e })?;

    let current_outfit_version = match sqlite_get_current_outfit_version(&db_conn).await {
        Ok(version) => version,
        Err(e) => {
            warn!("Failed to get current outfit version {:?}", e);
            0
        }
    };

    let avatar_current_version =
        match sqlite_get_current_avatar_version(&db_conn, agent_id.to_string()).await {
            Ok(version) => version,
            Err(e) => {
                warn!("Failed to get current avatar version {:?}", e);
                0
            }
        };

    sqlite_insert_avatar(&db_conn, agent_id, current_outfit_version)
        .await
        .map_err(|e| OutfitError::AvatarCacheFailure { error: e })?;

    let avatar = if current_outfit_version == avatar_current_version {
        Some(
            sqlite_get_avatar(&db_conn, agent_id)
                .await
                .map_err(|e| OutfitError::AvatarDataRetrieveFailure { error: e })?,
        )
    } else {
        None
    };

    Ok((avatar, outfit))
}

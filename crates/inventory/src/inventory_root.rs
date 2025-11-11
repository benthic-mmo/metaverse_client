use crate::errors::InventoryError;
use metaverse_messages::http::folder_request::FolderRequest;
use metaverse_messages::http::folder_types::{Category, Folder};
use metaverse_messages::utils::item_metadata::ItemMetadata;
use metaverse_messages::utils::object_types::ObjectType;
use serde_llsd_benthic::from_str;
use sqlx::{Row, SqlitePool};
use std::time::SystemTime;
use uuid::Uuid;

pub async fn refresh_inventory(
    pool: &SqlitePool,
    folder_request: FolderRequest,
    server_endpoint: String,
) -> Result<(), InventoryError> {
    use awc::Client;
    use std::collections::HashSet;

    async fn refresh_recursive(
        pool: &SqlitePool,
        folder_request: FolderRequest,
        server_endpoint: &str,
        visited: &mut HashSet<Uuid>,
    ) -> Result<(), InventoryError> {
        let client = Client::default();
        let url = server_endpoint.to_string();

        let mut response = client
            .post(url)
            .insert_header(("Content-Type", "application/llsd+xml"))
            .send_body(folder_request.to_llsd()?)
            .await?;

        let body_bytes = response.body().await?;
        let data = String::from_utf8_lossy(&body_bytes);

        if data.is_empty() || data == "<llsd><map><key>folders</key><array /></map></llsd>" {
            return Err(InventoryError::Error("Empty endpoint response".to_string()));
        }

        let parsed_data = from_str(&data)?;
        let folders = Folder::from_llsd(parsed_data)?;

        for folder in folders {
            if !visited.insert(folder.folder_id) {
                continue; // skip already processed
            }

            insert_folder(pool, &folder).await?;
            insert_items(pool, &folder.folder_id, &folder.items).await?;
            insert_categories(pool, &folder.folder_id, &folder.categories).await?;

            for category in &folder.categories {
                let category_request = FolderRequest {
                    folder_id: category.category_id,
                    owner_id: folder.owner_id,
                    fetch_items: true,
                    fetch_folders: true,
                    sort_order: 0,
                };
                Box::pin(refresh_recursive(
                    pool,
                    category_request,
                    server_endpoint,
                    visited,
                ))
                .await?;
            }
        }

        Ok(())
    }

    let mut visited = HashSet::new();
    refresh_recursive(pool, folder_request, &server_endpoint, &mut visited).await
}

pub async fn insert_folder(pool: &SqlitePool, folder: &Folder) -> Result<(), InventoryError> {
    let folder_id = folder.folder_id.to_string();
    let owner_id = folder.owner_id.to_string();
    let agent_id = folder.agent_id.to_string();

    sqlx::query(
        r#"
        INSERT OR REPLACE INTO folders (id, owner_id, agent_id, descendent_count, version)
        VALUES (?, ?, ?, ?, ?)
        "#,
    )
    .bind(&folder_id)
    .bind(&owner_id)
    .bind(&agent_id)
    .bind(folder.descendent_count)
    .bind(folder.version)
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn insert_categories(
    pool: &SqlitePool,
    folder_id: &Uuid,
    categories: &[Category],
) -> Result<(), InventoryError> {
    let folder_id = folder_id.to_string();
    for category in categories {
        let category_id = category.category_id.to_string();
        let type_default = category.type_default.to_string();

        sqlx::query(
            r#"
            INSERT OR REPLACE INTO categories (folder_id, name, id, type_default, version)
            VALUES (?, ?, ?, ?, ?)
            "#,
        )
        .bind(&folder_id)
        .bind(&category.name)
        .bind(&category_id)
        .bind(&type_default)
        .bind(category.version)
        .execute(pool)
        .await?;
    }
    Ok(())
}

pub async fn insert_items(
    pool: &SqlitePool,
    folder_id: &Uuid,
    items: &[ItemMetadata],
) -> Result<(), InventoryError> {
    let folder_id = folder_id.to_string();

    for item in items {
        let item_id = item.item_id.to_string();
        let asset_id = item.asset_id.to_string();
        let parent_id = item.parent_id.to_string();
        let created_at = item
            .created_at
            .duration_since(SystemTime::UNIX_EPOCH)?
            .as_secs() as i64;

        let owner_id = item.permissions.owner_id.to_string();
        let group_id = item.permissions.group_id.to_string();
        let creator_id = item.permissions.creator_id.to_string();
        let last_owner_id = item.permissions.last_owner_id.map(|id| id.to_string());
        sqlx::query(
            r#"
            INSERT OR REPLACE INTO items (
                name, item_id, asset_id, parent_id, description, created_at, inventory_type, flags, item_type, folder_id,
                owner_id, group_id, creator_id, base_mask, everyone_mask, group_mask, next_owner_mask, owner_mask, is_owner_group, last_owner_id,
                sale_type, price, ownership_cost
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#
        )
        .bind(&item.name)
        .bind(&item_id)
        .bind(&asset_id)
        .bind(&parent_id)
        .bind(&item.description)
        .bind(created_at)
        .bind(item.inventory_type.to_string())
        .bind(item.flags)
        .bind(item.item_type.to_string())
        .bind(&folder_id)
        .bind(&owner_id)
        .bind(&group_id)
        .bind(&creator_id)
        .bind(item.permissions.base_mask)
        .bind(item.permissions.everyone_mask)
        .bind(item.permissions.group_mask)
        .bind(item.permissions.next_owner_mask)
        .bind(item.permissions.owner_mask)
        .bind(item.permissions.is_owner_group)
        .bind(&last_owner_id)
        .bind(item.sale_info.sale_type.to_string())
        .bind(item.sale_info.price)
        .bind(item.sale_info.ownership_cost)
        .execute(pool)
        .await?;
    }

    Ok(())
}

pub async fn get_object_type_by_id(
    pool: &SqlitePool,
    item_id: &Uuid,
) -> Result<Option<(ObjectType, Uuid, String)>, InventoryError> {
    let item_id_str = item_id.to_string();
    let row = sqlx::query(
        r#"
        SELECT item_type, asset_id, name
        FROM items
        WHERE item_id = ?
        "#,
    )
    .bind(&item_id_str)
    .fetch_optional(pool)
    .await?;

    if let Some(row) = row {
        let item_type_str: String = row.try_get("item_type")?;
        let item_type = ObjectType::from(item_type_str.as_str());

        let asset_id_str: String = row.try_get("asset_id")?;
        let asset_id = Uuid::parse_str(&asset_id_str)?;

        let name: String = row.try_get("name")?;
        Ok(Some((item_type, asset_id, name)))
    } else {
        Ok(None)
    }
}

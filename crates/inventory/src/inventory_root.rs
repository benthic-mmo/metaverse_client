use std::time::SystemTime;
use std::{collections::HashMap, fs::create_dir_all, path::PathBuf};

use log::{info, warn};
use metaverse_messages::http::folder_request::FolderRequest;
use metaverse_messages::http::folder_types::{Category, Folder, FolderNode};
use metaverse_messages::utils::item_metadata::ItemMetadata;
use rusqlite::{params, Connection};
use serde_llsd_benthic::from_str;
use uuid::Uuid;
use crate::errors::InventoryError;

pub async fn refresh_inventory_2(
    conn: &mut Connection,
    folder_request: FolderRequest,
    server_endpoint: String,
) -> Result<(), InventoryError> {
    use std::collections::HashSet;
    

    // Define the recursive helper
    async fn refresh_recursive(
        conn: &mut Connection,
        folder_request: FolderRequest,
        server_endpoint: &str,
        visited: &mut HashSet<Uuid>,
    ) -> Result<(), InventoryError> {
        use awc::Client;

        let client = Client::default();
        let url = server_endpoint.to_string();

        let mut response = client
            .post(url)
            .insert_header(("Content-Type", "application/llsd+xml"))
            .send_body(folder_request.to_llsd()?)
            .await?;

        let body_bytes = response
            .body()
            .await?;

        let data = String::from_utf8_lossy(&body_bytes);
        let parsed_data = from_str(&data)?;
        let folders = Folder::from_llsd(parsed_data)?;

        for folder in folders {
            if !visited.insert(folder.folder_id) {
                // Already processed, skip to prevent infinite recursion
                continue;
            }

            insert_folder(conn, &folder)?;
            insert_items(conn, &folder.folder_id, &folder.items)?;
            insert_categories(conn, &folder.folder_id, &folder.categories)?;

            for category in &folder.categories {
                let category_request = FolderRequest {
                    folder_id: category.category_id,
                    owner_id: folder.owner_id,
                    fetch_items: true,
                    fetch_folders: true,
                    sort_order: 0,
                };

                // Box the recursive async call
                Box::pin(refresh_recursive(
                    conn,
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
    refresh_recursive(conn, folder_request, &server_endpoint, &mut visited).await
}


fn insert_folder(conn: &mut Connection, folder: &Folder) -> Result<(), InventoryError> {
    conn.execute(
        "INSERT OR REPLACE INTO folders (id, owner_id, agent_id, descendent_count, version)
         VALUES (?1, ?2, ?3, ?4, ?5)",
        params![
            folder.folder_id.to_string(),
            folder.owner_id.to_string(),
            folder.agent_id.to_string(),
            folder.descendent_count,
            folder.version,
        ],
    )?;
    Ok(())
}

fn insert_categories(conn: &mut Connection, folder_id:&Uuid, categories: &[Category]) -> Result<(), InventoryError>{
    for category in categories{
        conn.execute(
            "INSERT OR REPLACE INTO categories (folder_id, name, id, type_default, version)
            VALUES (?1, ?2, ?3, ?4, ?5)",
            params![
                folder_id.to_string(), 
                category.name,
                category.category_id.to_string(),
                category.type_default.to_string(), 
                category.version,

            ]
        )?;
    }
    Ok(())
}

fn insert_items(
    conn: &mut Connection,
    folder_id: &Uuid,
    items: &[ItemMetadata],
) -> Result<(), InventoryError> {
    for item in items {
        conn.execute(
            "INSERT OR REPLACE INTO items (
             name, item_id, asset_id, parent_id, description, created_at, inventory_type, flags, item_type, folder_id,
             owner_id, group_id, creator_id, base_mask, everyone_mask, group_mask, next_owner_mask, owner_mask, is_owner_group, last_owner_id, 
             sale_type, price, ownership_cost
            )
             VALUES (
                ?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10,
                ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18, ?19, ?20, 
                ?21, ?22, ?23
                )",
            params![
                item.name,
                item.item_id.to_string(),
                item.asset_id.to_string(),
                item.parent_id.to_string(),
                item.description,
                item.created_at.duration_since(SystemTime::UNIX_EPOCH).unwrap().as_secs(),
                item.inventory_type.to_string(), 
                item.flags,
                item.item_type.to_string(),
                folder_id.to_string(),

                item.permissions.owner_id.to_string(), 
                item.permissions.group_id.to_string(), 
                item.permissions.creator_id.to_string(), 
                item.permissions.base_mask,
                item.permissions.everyone_mask,
                item.permissions.group_mask,
                item.permissions.next_owner_mask,
                item.permissions.owner_mask,
                item.permissions.is_owner_group,
                item.permissions.last_owner_id.map(|id| id.to_string()),

                item.sale_info.sale_type.to_string(), 
                item.sale_info.price,
                item.sale_info.ownership_cost,



            ],
        )?;

    }
    Ok(())
}

pub async fn refresh_inventory(
    folder_request: FolderRequest,
    server_endpoint: String,
    inventory_current_dir: PathBuf,
) -> Result<FolderNode, InventoryError> {
    println!("inventory_current_dir: {:?}", inventory_current_dir);
    let client = awc::Client::default();
    let url = server_endpoint.to_string();
    let data = match client
        .post(url)
        .insert_header(("Content-Type", "application/llsd+xml"))
        .send_body(folder_request.to_llsd()?)
        .await
    {
        Ok(mut response) => match response.body().await {
            Ok(body_bytes) => body_bytes,
            Err(e) => {
                return Err(InventoryError::Error(format!(
                    "Failed to read body: {:?}",
                    e
                )));
            }
        },
        Err(e) => {
            return Err(InventoryError::Error(format!(
                "Failed to send with: {:?}",
                e
            )));
        }
    };

    let node = Box::pin(establish_inventory_dirs(
        data.as_ref(),
        server_endpoint,
        inventory_current_dir,
    ))
    .await?;
    Ok(node)
}

async fn establish_inventory_dirs(
    data: &[u8],
    server_endpoint: String,
    mut current_dir: PathBuf,
) -> Result<FolderNode, InventoryError> {
    let data = String::from_utf8_lossy(data);
    let parsed_data = from_str(&data)?;
    let folders = Folder::from_llsd(parsed_data)?;

    if let Some(data_dir) = dirs::data_dir() {
        let local_share_dir = data_dir.join("benthic");
        if !local_share_dir.exists() {
            if let Err(e) = create_dir_all(&local_share_dir) {
                warn!("Failed to create local share benthic: {}", e);
            };
            info!("Created Directory: {:?}", local_share_dir);
        }
        let inventory_root = local_share_dir.join("inventory");
        if !inventory_root.exists() {
            if let Err(e) = create_dir_all(&inventory_root) {
                warn!("Failed to create inventory {:?}", e);
            };
            info!("Created Directory: {:?}", inventory_root);
        }

        if let Some(folder) = folders.into_iter().next() {
            current_dir.push(folder.folder_id.to_string());
            let inventory_current_dir = inventory_root.join(current_dir.clone());
            let mut root_node = FolderNode {
                folder: folder.clone(),
                path: inventory_current_dir,
                children: HashMap::new(),
            };

            let mut children_nodes = HashMap::new();
            for category in folder.categories {
                let category_request = FolderRequest {
                    folder_id: category.category_id,
                    owner_id: folder.owner_id,
                    fetch_items: true,
                    fetch_folders: true,
                    sort_order: 0,
                };
                let child_node = refresh_inventory(
                    category_request,
                    server_endpoint.clone(),
                    current_dir.clone(),
                )
                .await?;
                children_nodes.insert(category.type_default, child_node);
            }
            root_node.children.extend(children_nodes);
            return Ok(root_node);
        }
    }
    Err(InventoryError::Error("No folders found".to_string()))
}

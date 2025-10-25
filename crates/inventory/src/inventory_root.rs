use std::{collections::HashMap, fs::create_dir_all, path::PathBuf};

use log::{info, warn};
use metaverse_messages::http::folder_types::{Folder, FolderNode};
use serde_llsd_benthic::from_str;
use serde_llsd_benthic::{ser::xml, LLSDValue};
use uuid::Uuid;

use crate::errors::InventoryError;

#[derive(Debug)]
pub struct FolderRequest {
    pub folder_id: Uuid,
    pub owner_id: Uuid,
    pub fetch_folders: bool,
    pub fetch_items: bool,
    pub sort_order: u8,
}
impl FolderRequest {
    pub fn to_llsd(&self) -> Result<String, InventoryError> {
        let mut map = HashMap::new();
        map.insert("folder_id".to_string(), LLSDValue::UUID(self.folder_id));
        map.insert("owner_id".to_string(), LLSDValue::UUID(self.owner_id));
        map.insert(
            "fetch_folders".to_string(),
            LLSDValue::Boolean(self.fetch_folders),
        );
        map.insert(
            "fetch_items".to_string(),
            LLSDValue::Boolean(self.fetch_items),
        );
        map.insert(
            "sort_order".to_string(),
            LLSDValue::Integer(self.sort_order as i32),
        );

        let folder_data = LLSDValue::Map(map);
        let folders_array = LLSDValue::Array(vec![folder_data]);

        let mut outer_map = HashMap::new();
        outer_map.insert("folders".to_string(), folders_array);
        let put_xml = LLSDValue::Map(outer_map);
        let xml = xml::to_string(&put_xml, false)?;
        Ok(xml)
    }
}

pub async fn refresh_inventory(
    folder_request: FolderRequest,
    server_endpoint: String,
    inventory_current_dir: PathBuf,
) -> Result<FolderNode, InventoryError> {
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

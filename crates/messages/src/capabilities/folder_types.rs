use crate::capabilities::item_types::Item;
use actix::Message;
use serde_llsd::LLSDValue;
use std::{collections::HashMap, path::PathBuf};
use uuid::Uuid;

use crate::utils::object_types::ObjectType;

#[derive(Debug, Clone, Message)]
#[rtype(result = "()")]
/// The data structure for storing the folder tree
pub struct FolderNode {
    /// The folder object itself that contains the owner id, items, and etc
    pub folder: Folder,
    /// The child folders of the folder
    pub children: HashMap<ObjectType, FolderNode>,
    /// the path of the folder on the disk
    pub path: PathBuf,
}

#[derive(Debug, Clone)]
/// Category folders. Contains information about cateogies under the root folder.
pub struct Category {
    /// Name of the category
    pub name: String,
    /// UUID of the folder the category describes
    pub category_id: Uuid,
    /// Default type contained in the folder
    pub type_default: ObjectType,
    /// version. Used to trigger redownloading on update
    pub version: i32,
}
impl Category {
    /// converts a Category parsed with LLSD to a local type
    pub fn from_llsd(category: &LLSDValue) -> std::io::Result<Category> {
        if let Some(category_info) = category.as_map() {
            let name = match category_info.get("name") {
                Some(LLSDValue::String(name)) => name.to_string(),
                _ => "".to_string(),
            };
            let category_id = match category_info.get("category_id") {
                Some(LLSDValue::UUID(category_id)) => *category_id,
                _ => Uuid::nil(),
            };
            let type_default = match category_info.get("type_default") {
                Some(LLSDValue::Integer(type_default)) => {
                    ObjectType::from_bytes(&if *type_default >= 0 {
                        *type_default as u8
                    } else {
                        99
                    })
                }
                _ => ObjectType::Unknown,
            };
            let version = match category_info.get("version") {
                Some(LLSDValue::Integer(version)) => *version,
                _ => 0,
            };
            Ok(Category {
                name,
                category_id,
                type_default,
                version,
            })
        } else {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Missing or invalid category data",
            ));
        }
    }
}
#[derive(Debug, Clone)]
/// The folder struct. Contains things like items and categories
pub struct Folder {
    /// The ID of the folder. Also the name of the real folder on disk.
    pub folder_id: Uuid,
    /// Owner of the folder
    pub owner_id: Uuid,
    /// Agent id of the user
    pub agent_id: Uuid,
    /// Version of the folder. Used to trigger redownloading on update.
    pub version: i32,
    /// Number of descendents the folder has
    pub descendent_count: i32,
    /// Items contained within the folder
    pub items: Vec<Item>,
    /// sub-folders within the folder
    pub categories: Vec<Category>,
}
impl Folder {
    /// When the server responds with
    pub fn from_llsd(parsed_data: LLSDValue) -> std::io::Result<Vec<Self>> {
        let mut folders_vec = Vec::new();
        if let Ok(data) = parsed_data.into_map() {
            if let Some(folders) = data.get("folders") {
                if let Some(folders) = folders.as_array() {
                    for folder in folders {
                        if let Some(folder_data) = folder.as_map() {
                            let folder_id = match folder_data.get("folder_id") {
                                Some(LLSDValue::UUID(id)) => *id,
                                _ => Uuid::nil(),
                            };
                            let owner_id = match folder_data.get("owner_id") {
                                Some(LLSDValue::UUID(id)) => *id,
                                _ => Uuid::nil(),
                            };
                            let descendent_count = match folder_data.get("descendents") {
                                Some(LLSDValue::Integer(int)) => *int,
                                _ => 0,
                            };
                            let version = match folder_data.get("version") {
                                Some(LLSDValue::Integer(int)) => *int,
                                _ => 0,
                            };

                            let agent_id = match folder_data.get("agent_id") {
                                Some(LLSDValue::UUID(id)) => *id,
                                _ => Uuid::nil(),
                            };
                            let mut items_vec = Vec::new();
                            match folder_data.get("items") {
                                Some(items) => {
                                    if let Some(items) = items.as_array() {
                                        for item in items {
                                            items_vec.push(Item::from_llsd(item)?)
                                        }
                                    }
                                }
                                _ => {}
                            };
                            let mut category_vec = Vec::new();
                            match folder_data.get("categories") {
                                Some(categories) => {
                                    if let Some(categories) = categories.as_array() {
                                        for category in categories {
                                            category_vec.push(Category::from_llsd(category)?);
                                        }
                                    }
                                }
                                _ => {}
                            };
                            folders_vec.push(Folder {
                                folder_id,
                                owner_id,
                                descendent_count,
                                version,
                                agent_id,
                                items: items_vec,
                                categories: category_vec,
                            });
                        }
                    }
                }
            }
        }
        Ok(folders_vec)
    }
}

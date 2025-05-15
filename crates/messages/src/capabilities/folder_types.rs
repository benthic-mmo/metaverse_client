use std::{collections::HashMap, path::PathBuf};

use actix::Message;
use serde_llsd::LLSDValue;
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
    pub items: Option<Vec<LLSDValue>>,
    /// sub-folders within the folder
    pub categories: Vec<Category>,
}
impl Folder {
    /// When the server responds with
    pub fn from_llsd(data: &[u8]) -> std::io::Result<Vec<Self>> {
        let str_data = match std::str::from_utf8(&data) {
            Ok(str) => str,
            _ => {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    "Unable to parse string",
                ));
            }
        };
        let parsed_data = match serde_llsd::from_str(str_data) {
            Ok(str) => str,
            _ => {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    "Unable to parse string",
                ));
            }
        };
        let mut folders_vec = Vec::new();
        if let Ok(data) = parsed_data.into_map() {
            if let Some(folders) = data.get("folders") {
                if let Some(folders) = folders.as_array() {
                    for folder in folders {
                        if let Some(folder_data) = folder.as_map() {
                            let folder_id = match folder_data.get("folder_id") {
                                Some(LLSDValue::UUID(id)) => *id,
                                _ => {
                                    return Err(std::io::Error::new(
                                        std::io::ErrorKind::InvalidData,
                                        "Missing or invalid folder ID",
                                    ));
                                }
                            };
                            let owner_id = match folder_data.get("owner_id") {
                                Some(LLSDValue::UUID(id)) => *id,
                                _ => {
                                    return Err(std::io::Error::new(
                                        std::io::ErrorKind::InvalidData,
                                        "Missing or invalid owner ID",
                                    ));
                                }
                            };
                            let descendent_count = match folder_data.get("descendents") {
                                Some(LLSDValue::Integer(int)) => *int,
                                _ => {
                                    return Err(std::io::Error::new(
                                        std::io::ErrorKind::InvalidData,
                                        "Missing or invalid descendent count",
                                    ));
                                }
                            };
                            let version = match folder_data.get("version") {
                                Some(LLSDValue::Integer(int)) => *int,
                                _ => {
                                    return Err(std::io::Error::new(
                                        std::io::ErrorKind::InvalidData,
                                        "Missing or invalid folder version",
                                    ));
                                }
                            };

                            let agent_id = match folder_data.get("agent_id") {
                                Some(LLSDValue::UUID(id)) => *id,
                                _ => {
                                    return Err(std::io::Error::new(
                                        std::io::ErrorKind::InvalidData,
                                        "Missing or invalid agent ID",
                                    ));
                                }
                            };
                            let items = match folder_data.get("items") {
                                Some(items) => items.as_array().cloned(),
                                _ => {
                                    return Err(std::io::Error::new(
                                        std::io::ErrorKind::InvalidData,
                                        "Missing or invalid items",
                                    ));
                                }
                            };
                            let mut category_vec = Vec::new();
                            match folder_data.get("categories") {
                                Some(categories) => {
                                    if let Some(categories) = categories.as_array() {
                                        for category in categories {
                                            if let Some(category_info) = category.as_map() {
                                                let name = match category_info.get("name") {
                                                    Some(LLSDValue::String(name)) => {
                                                        name.to_string()
                                                    }
                                                    _ => {
                                                        return Err(std::io::Error::new(
                                                            std::io::ErrorKind::InvalidData,
                                                            "Missing or invalid category name",
                                                        ));
                                                    }
                                                };
                                                let category_id =
                                                    match category_info.get("category_id") {
                                                        Some(LLSDValue::UUID(category_id)) => {
                                                            *category_id
                                                        }
                                                        _ => {
                                                            return Err(std::io::Error::new(
                                                                std::io::ErrorKind::InvalidData,
                                                                "Missing or invalid category id",
                                                            ));
                                                        }
                                                    };
                                                let type_default =
                                                    match category_info.get("type_default") {
                                                        Some(LLSDValue::Integer(type_default)) => {
                                                            ObjectType::from_bytes(
                                                                &if *type_default >= 0 {
                                                                    *type_default as u8
                                                                } else {
                                                                    99
                                                                },
                                                            )
                                                        }
                                                        _ => {
                                                            return Err(std::io::Error::new(
                                                                std::io::ErrorKind::InvalidData,
                                                                "Missing or invalid category type",
                                                            ));
                                                        }
                                                    };
                                                let version = match category_info.get("version") {
                                                    Some(LLSDValue::Integer(version)) => *version,
                                                    _ => {
                                                        return Err(std::io::Error::new(
                                                            std::io::ErrorKind::InvalidData,
                                                            "Missing or invalid category version",
                                                        ));
                                                    }
                                                };
                                                category_vec.push(Category {
                                                    name,
                                                    category_id,
                                                    type_default,
                                                    version,
                                                });
                                            }
                                        }
                                    }
                                }
                                _ => {
                                    return Err(std::io::Error::new(
                                        std::io::ErrorKind::InvalidData,
                                        "Missing or invalid items",
                                    ));
                                }
                            };
                            folders_vec.push(Folder {
                                folder_id,
                                owner_id,
                                descendent_count,
                                version,
                                agent_id,
                                items,
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

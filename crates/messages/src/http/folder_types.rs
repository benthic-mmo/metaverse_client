use serde_llsd_benthic::LLSDValue;
use uuid::Uuid;

use crate::{
    errors::ParseError,
    parse,
    utils::{item_metadata::ItemMetadata, object_types::ObjectType},
};

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
    /// Converts a Category parsed with LLSD to a local type
    pub fn from_llsd(category: &LLSDValue) -> Result<Category, ParseError> {
        let category_info = category
            .as_map()
            .ok_or_else(|| ParseError::message("Expected category to be a map"))?;

        let name = match category_info.get("name") {
            Some(LLSDValue::String(name)) => name.clone(),
            Some(_) => return Err(ParseError::message("Field 'name' is not a string"))?,
            None => return Err(ParseError::message("Missing field 'name'"))?,
        };

        let category_id = match category_info.get("category_id") {
            Some(LLSDValue::UUID(id)) => *id,
            Some(_) => {
                return Err(ParseError::message("Field 'category_id' is not a UUID"))?;
            }
            None => return Err(ParseError::message("Missing field 'category_id'"))?,
        };

        let type_default = match category_info.get("type_default") {
            Some(LLSDValue::Integer(i)) if *i >= 0 => ObjectType::from_bytes(&(*i as u8)),
            Some(LLSDValue::Integer(_)) => ObjectType::Unknown,
            Some(_) => {
                return Err(ParseError::message(
                    "Field 'type_default' is not an integer",
                ))?;
            }
            None => ObjectType::Unknown,
        };

        let version = match category_info.get("version") {
            Some(LLSDValue::Integer(v)) => *v,
            Some(_) => {
                return Err(ParseError::message("Field 'version' is not an integer"))?;
            }
            None => 0,
        };
        Ok(Category {
            name,
            category_id,
            type_default,
            version,
        })
    }
}

#[derive(Debug, Clone)]
/// contains success and failure for retrieving folders from the capability endpoint
pub enum FolderResult {
    /// Folder retrieved successfully  
    Success(Folder),
    /// Failed to retrieve folder
    Failure(FolderError),
}

#[derive(Debug, Clone)]
/// Describes the folder error returned from the capability endpoint
pub struct FolderError {
    /// ID of failed folder
    pub folder_id: Uuid,
    /// Error string of failed folder
    pub error: String,
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
    pub items: Vec<ItemMetadata>,
    /// sub-folders within the folder
    pub categories: Vec<Category>,
}

impl Folder {
    /// convert from LLSD to a vector of folders
    pub fn from_llsd(parsed_data: LLSDValue) -> Result<Vec<Self>, ParseError> {
        let mut folders_vec = Vec::new();

        let data = parsed_data
            .into_map()
            .map_err(|_| ParseError::message("Expected top-level map"))?;

        let folders = data
            .get("folders")
            .and_then(|v| v.as_array())
            .ok_or(ParseError::message("Missing or invalid 'folders' array"))?;

        for folder in folders {
            let folder_data = folder
                .as_map()
                .ok_or(ParseError::message("Folder entry is not a map"))?;

            let folder_id = match folder_data.get("folder_id") {
                Some(LLSDValue::UUID(id)) => *id,
                Some(_) => {
                    return Err(ParseError::message("Field 'folder_id' is not a UUID"));
                }
                None => return Err(ParseError::message("Missing field 'folder_id'")),
            };

            let owner_id = match folder_data.get("owner_id") {
                Some(LLSDValue::UUID(id)) => *id,
                Some(_) => {
                    return Err(ParseError::message("Field 'owner_id' is not a UUID"));
                }
                None => return Err(ParseError::message("Missing field 'owner_id'")),
            };

            let descendent_count = match folder_data.get("descendents") {
                Some(LLSDValue::Integer(int)) => *int,
                Some(_) => {
                    return Err(ParseError::message("Field 'descendents' is not an integer"));
                }
                None => 0,
            };

            let version = match folder_data.get("version") {
                Some(LLSDValue::Integer(int)) => *int,
                Some(_) => {
                    return Err(ParseError::message("Field 'version' is not an integer"));
                }
                None => 0,
            };

            let agent_id = match folder_data.get("agent_id") {
                Some(LLSDValue::UUID(id)) => *id,
                Some(_) => {
                    return Err(ParseError::message("Field 'agent_id' is not a UUID"));
                }
                None => return Err(ParseError::message("Missing field 'agent_id'")),
            };

            // Parse items
            let mut items_vec = Vec::new();
            if let Some(items) = folder_data.get("items").and_then(|v| v.as_array()) {
                for item in items {
                    let item = parse!(ItemMetadata::from_llsd(item))?;
                    items_vec.push(item);
                }
            }

            // Parse categories
            let mut category_vec = Vec::new();
            if let Some(categories) = folder_data.get("categories").and_then(|v| v.as_array()) {
                for category in categories {
                    let category = Category::from_llsd(category)?;
                    category_vec.push(category);
                }
            }

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

        Ok(folders_vec)
    }
}

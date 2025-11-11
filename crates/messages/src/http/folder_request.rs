use std::collections::HashMap;

use crate::errors::ParseError;
use serde_llsd_benthic::{ser::xml, LLSDValue};
use uuid::Uuid;

#[derive(Debug)]
/// The struct to define a request for a folder
pub struct FolderRequest {
    /// ID of the folder being requested
    pub folder_id: Uuid,
    /// owner of the folder being requested
    pub owner_id: Uuid,
    /// bool to fetch folders
    pub fetch_folders: bool,
    /// bool to fetch items
    pub fetch_items: bool,
    /// sort order
    pub sort_order: u8,
}
impl FolderRequest {
    /// convert folder request to LLSD for sending to the capabilty endpoint
    pub fn to_llsd(&self) -> Result<String, ParseError> {
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

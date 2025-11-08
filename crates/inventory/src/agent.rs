use std::str::FromStr;

use metaverse_messages::utils::object_types::ObjectType;
use rusqlite::Connection;
use uuid::Uuid;

use crate::errors::InventoryError;

pub fn get_current_outfit(
    conn: &Connection,
) -> Result<Vec<(String, Uuid, ObjectType)>, InventoryError> {
    let folder_id: String = conn.query_row(
        "SELECT id
         FROM categories
         WHERE name = 'Current Outfit'
         LIMIT 1", // just in case there are multiple
        [],
        |row| row.get(0),
    )?;
    let mut item = conn.prepare(
        "SELECT name, item_id, item_type 
         FROM items
         WHERE folder_id = ?1",
    )?;
    let rows = item
        .query_map([folder_id], |row| {
            let name: String = row.get(0)?;
            let item_id_str: String = row.get(1)?;
            let item_type_str: String = row.get(2)?;

            let item_id = Uuid::from_str(&item_id_str).map_err(|e| {
                rusqlite::Error::FromSqlConversionFailure(
                    1,
                    rusqlite::types::Type::Text,
                    Box::new(e),
                )
            })?;
            let item_type = ObjectType::from(item_type_str);
            Ok((name, item_id, item_type))
        })?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(rows)
}

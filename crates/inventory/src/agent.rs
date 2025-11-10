use std::str::FromStr;

use metaverse_messages::utils::object_types::ObjectType;
use rusqlite::Connection;
use uuid::Uuid;

use crate::errors::InventoryError;

pub fn get_agent_outfit(
    conn: &Connection,
    agent_id: Uuid,
) -> Result<Vec<(String, Uuid, Uuid, ObjectType)>, InventoryError> {
    let mut stmt = conn.prepare(
        "SELECT name, item_id, asset_id, item_type
         FROM items
         WHERE folder_id = ?1",
    )?;
    let rows = stmt
        .query_map([agent_id.to_string()], |row| {
            let name: String = row.get(0)?;
            let item_id_str: String = row.get(1)?;
            let asset_id_str: String = row.get(2)?;
            let item_type_str: String = row.get(3)?;
            let item_type = ObjectType::from(item_type_str.clone());

            let item_id = Uuid::from_str(&item_id_str).map_err(|e| {
                rusqlite::Error::FromSqlConversionFailure(
                    1,
                    rusqlite::types::Type::Text,
                    Box::new(e),
                )
            })?;
            let asset_id = Uuid::from_str(&asset_id_str).map_err(|e| {
                rusqlite::Error::FromSqlConversionFailure(
                    1,
                    rusqlite::types::Type::Text,
                    Box::new(e),
                )
            })?;
            Ok((name, item_id, asset_id, item_type))
        })?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(rows)
}

pub fn get_current_outfit(
    conn: &Connection,
) -> Result<Vec<(String, Uuid, Uuid, ObjectType)>, InventoryError> {
    let folder_id: String = conn.query_row(
        "SELECT id
         FROM categories
         WHERE name = 'Current Outfit'
         LIMIT 1", // just in case there are multiple
        [],
        |row| row.get(0),
    )?;
    let mut item = conn.prepare(
        "SELECT name, item_id, asset_id, item_type 
         FROM items
         WHERE folder_id = ?1",
    )?;
    let rows = item
        .query_map([folder_id], |row| {
            let mut name: String = row.get(0)?;
            let mut item_id_str: String = row.get(1)?;
            let mut asset_id_str: String = row.get(2)?;
            let mut item_type_str: String = row.get(3)?;
            let item_type = ObjectType::from(item_type_str.clone());

            // If the item is a link, retrieve and shadow the actual linked item
            // the linked item's asset_id is the same as the real object's item_id.
            if item_type == ObjectType::Link {
                let mut stmt = conn.prepare(
                    "SELECT name, item_id, asset_id, item_type
                     FROM items
                     WHERE item_id = ?1",
                )?;

                let mut rows = stmt.query_map([asset_id_str.clone()], |r| {
                    Ok((
                        r.get::<_, String>(0)?,
                        r.get::<_, String>(1)?,
                        r.get::<_, String>(2)?,
                        r.get::<_, String>(3)?,
                    ))
                })?;

                if let Some(Ok((linked_name, linked_item_id, linked_asset_id, linked_item_type))) =
                    rows.next()
                {
                    // Shadow the original variables
                    name = linked_name;
                    item_id_str = linked_item_id;
                    asset_id_str = linked_asset_id;
                    item_type_str = linked_item_type;
                }
            }
            // ensure the ObjectType is correct, if it was a link
            let item_type = ObjectType::from(item_type_str);

            let item_id = Uuid::from_str(&item_id_str).map_err(|e| {
                rusqlite::Error::FromSqlConversionFailure(
                    1,
                    rusqlite::types::Type::Text,
                    Box::new(e),
                )
            })?;
            let asset_id = Uuid::from_str(&asset_id_str).map_err(|e| {
                rusqlite::Error::FromSqlConversionFailure(
                    1,
                    rusqlite::types::Type::Text,
                    Box::new(e),
                )
            })?;

            Ok((name, item_id, asset_id, item_type))
        })?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(rows)
}

use std::time::SystemTime;

use serde::{Deserialize, Serialize};
use serde_llsd_benthic::LLSDValue;
use uuid::Uuid;

use super::object_types::ObjectType;

#[derive(Debug, Clone, Serialize, Deserialize)]
/// Metadata shared by all opensimulator items
/// this is received by the FetchInventory endpoint, and can be used to retrieve additional items
/// data from the ViewerAsset endpoint.
pub struct ItemMetadata {
    /// Permissions attached to the item. Who can create, copy, transfer, sell etc.
    pub permissions: Permissions,
    /// name of the item
    pub name: String,
    /// Sale information of the item. Is it for sale, how much, etc.
    pub sale_info: SaleInfo,
    /// The ID of the item, used for retrieving from the ViewerAsset endpoint
    pub asset_id: Uuid,
    /// The parent object. If the object is the root, the id is zeroed out.
    pub parent_id: Uuid,
    /// The optional local parent ID. Some places in the code store the parent ID as a u32 to save
    /// space and not save the full UUID.
    pub parent_id_local: Option<u32>,
    /// local ID of the item
    pub item_id: Uuid,
    /// Description of the object.
    pub description: String,
    /// Unix timestamp for when the object was created
    pub created_at: std::time::SystemTime,
    /// Name of the  object
    pub inventory_type: i32,
    /// item flags
    pub flags: i32,
    /// type of the item. Clothes, tree, animation, etc.
    pub item_type: ObjectType,
}
impl Default for ItemMetadata {
    fn default() -> Self {
        Self {
            name: Default::default(),
            sale_info: Default::default(),
            permissions: Default::default(),
            asset_id: Default::default(),
            parent_id: Default::default(),
            parent_id_local: Default::default(),
            item_id: Default::default(),
            description: Default::default(),
            created_at: SystemTime::UNIX_EPOCH,
            inventory_type: 0,
            flags: 0,
            item_type: Default::default(),
        }
    }
}
impl ItemMetadata {
    /// Item metadata comes in through an unstructured newline seperated type when receiving item
    /// data from the ViewerAsset endpoint. This parses the item metadata as bytes.
    pub fn from_bytes(bytes: &[u8]) -> std::io::Result<Self> {
        let text = std::str::from_utf8(bytes)
            .map_err(|_| std::io::Error::new(std::io::ErrorKind::InvalidData, "Invalid UTF-8"))?;
        let mut lines = text.lines().map(str::trim);
        let mut metadata = ItemMetadata::default();

        let mut next_line = || {
            lines.next().ok_or_else(|| {
                std::io::Error::new(std::io::ErrorKind::UnexpectedEof, "Unexpected end of data")
            })
        };

        let _version = next_line()?.to_owned();
        let name = next_line()?.to_owned();
        metadata.name = name;

        next_line()?; // permissions 0
        next_line()?; // {
        next_line()?; // base_mask

        let parse_hex = |line: &str| {
            i32::from_str_radix(
                line.split('\t').nth(1).ok_or_else(|| {
                    std::io::Error::new(std::io::ErrorKind::InvalidData, "Missing hex field")
                })?,
                16,
            )
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string()))
        };

        let base_mask = parse_hex(next_line()?)?;
        let owner_mask = parse_hex(next_line()?)?;
        let group_mask = parse_hex(next_line()?)?;
        let everyone_mask = parse_hex(next_line()?)?;
        let next_owner_mask = parse_hex(next_line()?)?;

        let parse_uuid = |line: &str| {
            let id_str = line.split('\t').nth(1).ok_or_else(|| {
                std::io::Error::new(std::io::ErrorKind::InvalidData, "Missing UUID field")
            })?;
            Uuid::parse_str(id_str)
                .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string()))
        };

        let creator_id = parse_uuid(next_line()?)?;
        let owner_id = parse_uuid(next_line()?)?;
        let last_owner_id = Some(parse_uuid(next_line()?)?);
        let group_id = parse_uuid(next_line()?)?;

        let permissions = Permissions {
            base_mask,
            owner_mask,
            group_mask,
            everyone_mask,
            next_owner_mask,
            creator_id,
            last_owner_id,
            group_id,
            owner_id,
            is_owner_group: None,
        };
        metadata.permissions = permissions;

        next_line()?; // }
        next_line()?; // sale_info 0
        next_line()?; // {
        next_line()?; // sale_type

        let price = {
            let price_str = next_line()?.split('\t').nth(1).ok_or_else(|| {
                std::io::Error::new(std::io::ErrorKind::InvalidData, "Missing sale price")
            })?;
            parse_or_io(price_str)?
        };

        metadata.sale_info.price = price;

        Ok(metadata)
    }
    /// When receiving item metadata from the FetchInventoryDescendents endpoint. item metadadata
    /// is received in LLSD format.
    pub fn from_llsd(data: &LLSDValue) -> std::io::Result<Self> {
        if let Some(item) = data.as_map() {
            let asset_id = match item.get("asset_id") {
                Some(LLSDValue::UUID(asset_id)) => *asset_id,
                _ => Uuid::nil(),
            };
            let parent_id = match item.get("parent_id") {
                Some(LLSDValue::UUID(parent_id)) => *parent_id,
                _ => Uuid::nil(),
            };
            let item_id = match item.get("item_id") {
                Some(LLSDValue::UUID(item_id)) => *item_id,
                _ => Uuid::nil(),
            };

            let sale_info = match item.get("sale_info") {
                Some(sale) => SaleInfo::from_llsd(sale)?,
                _ => SaleInfo {
                    ..Default::default()
                },
            };

            let permissions = match item.get("permissions") {
                Some(permission) => Permissions::from_llsd(permission)?,
                _ => Permissions {
                    ..Default::default()
                },
            };

            let description = match item.get("desc") {
                Some(LLSDValue::String(description)) => description.to_string(),
                _ => "".to_string(),
            };
            let created_at = match item.get("created_at") {
                Some(LLSDValue::Integer(created_at)) => {
                    SystemTime::UNIX_EPOCH + std::time::Duration::from_secs(*created_at as u64)
                }
                _ => SystemTime::now(),
            };
            let name = match item.get("name") {
                Some(LLSDValue::String(name)) => name.to_string(),
                _ => "".to_string(),
            };
            let inventory_type = match item.get("inv_type") {
                Some(LLSDValue::Integer(asset_id)) => *asset_id,
                _ => 0,
            };
            let flags = match item.get("flags") {
                Some(LLSDValue::Integer(flags)) => *flags,
                _ => 0,
            };
            let item_type = match item.get("type") {
                Some(LLSDValue::Integer(item_type)) => {
                    ObjectType::from_bytes(&if *item_type >= 0 {
                        *item_type as u8
                    } else {
                        println!("WEIRD TYPE GOT {:?}", item_type);
                        99
                    })
                }
                _ => ObjectType::Unknown,
            };
            Ok(ItemMetadata {
                name,
                permissions,
                sale_info,
                asset_id,
                item_id,
                created_at,
                description,
                inventory_type,
                item_type,
                flags,
                parent_id,
                parent_id_local: None,
            })
        } else {
            Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Missing or invalid item metadata",
            ))
        }
    }
}
fn parse_or_io<T: std::str::FromStr>(s: &str) -> std::io::Result<T>
where
    T::Err: std::fmt::Display,
{
    s.parse::<T>()
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string()))
}
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
/// Information regarding the permissions the object has
pub struct Permissions {
    /// The UUID of the current owner of the object
    pub owner_id: Uuid,
    /// The group that the object belongs to. This allows permissions for objects to be handled on
    /// a user group level instead of an individual level.
    pub group_id: Uuid,
    /// The ID of the creator of the object
    pub creator_id: Uuid,
    /// The default permissions of the object that apply regardless of ownership or group.
    pub base_mask: i32,
    /// Permissions granted to all users. Broadest level of access.
    pub everyone_mask: i32,
    /// Permissions granted to users who belong to the specified group ID.
    pub group_mask: i32,
    /// Permissions granted to the next owner of the object, when it is transferred.
    pub next_owner_mask: i32,
    /// Permissions the current owner has on the object.
    pub owner_mask: i32,
    /// indicates if the owner is the group istelf, rather than an individual user
    pub is_owner_group: Option<bool>,
    /// Stores the UUID of the last owner
    pub last_owner_id: Option<Uuid>,
}
impl Permissions {
    /// convert LLSD values to permissions
    pub fn from_llsd(data: &LLSDValue) -> std::io::Result<Self> {
        if let Some(permissions) = data.as_map() {
            let owner_id = match permissions.get("owner_id") {
                Some(LLSDValue::UUID(owner_id)) => *owner_id,
                _ => Uuid::nil(),
            };
            let group_id = match permissions.get("group_id") {
                Some(LLSDValue::UUID(group_id)) => *group_id,
                _ => Uuid::nil(),
            };
            let creator_id = match permissions.get("creator_id") {
                Some(LLSDValue::UUID(creator_id)) => *creator_id,
                _ => Uuid::nil(),
            };
            let base_mask = match permissions.get("base_mask") {
                Some(LLSDValue::Integer(base_mask)) => *base_mask,
                _ => 0,
            };
            let everyone_mask = match permissions.get("everyone_mask") {
                Some(LLSDValue::Integer(everyone_mask)) => *everyone_mask,
                _ => 0,
            };
            let group_mask = match permissions.get("group_mask") {
                Some(LLSDValue::Integer(group_mask)) => *group_mask,
                _ => 0,
            };
            let next_owner_mask = match permissions.get("next_owner_mask") {
                Some(LLSDValue::Integer(next_owner_mask)) => *next_owner_mask,
                _ => 0,
            };
            let owner_mask = match permissions.get("owner_mask") {
                Some(LLSDValue::Integer(owner_mask)) => *owner_mask,
                _ => 0,
            };
            let is_owner_group = match permissions.get("is_owner_group") {
                Some(LLSDValue::Boolean(is_owner_group)) => Some(*is_owner_group),
                _ => None,
            };
            Ok(Permissions {
                owner_id,
                group_id,
                creator_id,
                base_mask,
                everyone_mask,
                group_mask,
                is_owner_group,
                next_owner_mask,
                owner_mask,
                last_owner_id: None,
            })
        } else {
            Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Missing or invalid image permission data",
            ))
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
/// information regarding the sale of the object. How it was sold and how much it costs.
pub struct SaleInfo {
    /// The type of sale
    pub sale_type: SaleType,
    /// The price of the object
    pub price: i32,
    /// Recurring cost that is associated with owning the object after the purchase.
    /// Functionally how much you are renting it for.
    pub ownership_cost: Option<i32>,
}
impl SaleInfo {
    /// Converts LLSD to a SaleInfo object
    pub fn from_llsd(data: &LLSDValue) -> std::io::Result<Self> {
        if let Some(sale_info) = data.as_map() {
            let sale_type = match sale_info.get("sale_type") {
                Some(LLSDValue::Integer(sale_type)) => SaleType::from_i32(*sale_type),
                _ => {
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::InvalidData,
                        "Missing or invalid sale type",
                    ));
                }
            };
            let price = match sale_info.get("sale_price") {
                Some(LLSDValue::Integer(price)) => *price,
                _ => {
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::InvalidData,
                        "Missing or invalid price",
                    ));
                }
            };
            Ok(SaleInfo {
                sale_type,
                price,
                ownership_cost: None,
            })
        } else {
            Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Missing or invalid image sale data",
            ))
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
/// Values to determine what the for sale state of an object is.
pub enum SaleType {
    /// Not for sale
    Not,
    /// The original item is for sale.
    Original,
    /// A copy of the original is for sale
    Copy,
    /// The contents inside the object are for sale
    Contents,

    #[default]
    /// unknown
    Unknown,
}
impl SaleType {
    /// Convert the SaleType from an i32
    pub fn from_i32(byte: i32) -> Self {
        match byte {
            0 => Self::Not,
            1 => Self::Original,
            2 => Self::Copy,
            3 => Self::Contents,
            _ => Self::Unknown,
        }
    }
    /// Convert the SaleType from a string
    pub fn from_string(str: &str) -> Self {
        match str {
            "not" | "Not" => SaleType::Not,
            "original" | "Original" => SaleType::Original,
            "copy" | "Copy" => SaleType::Copy,
            "contents" | "Contents" => SaleType::Contents,
            _ => SaleType::Unknown,
        }
    }
}

use std::time::SystemTime;

use serde_llsd::LLSDValue;
use uuid::Uuid;

use crate::utils::object_types::ObjectType;

#[derive(Debug, Clone)]
/// Inventory item struct for Opensimulator items
pub struct Item {
    /// The ID of the item, used for retrieving from the ViewerAsset endpoint
    pub asset_id: Uuid,
    /// The parent object. If the object is the root, the id is zeroed out.
    pub parent_id: Uuid,
    pub item_id: Uuid,
    /// information about the sale of an object
    pub sale_info: SaleInfo,
    /// permissions that the object has
    pub permissions: Permissions,
    /// Description of the object.
    pub description: String,
    /// Unix timestamp for when the object was created
    pub created_at: std::time::SystemTime,
    /// Name of the  object
    pub name: String,
    pub inventory_type: i32,
    pub flags: i32,
    pub item_type: ObjectType,
}
impl Default for Item {
    fn default() -> Self {
        Self {
            asset_id: Default::default(),
            parent_id: Default::default(),
            item_id: Default::default(),
            sale_info: Default::default(),
            permissions: Default::default(),
            description: Default::default(),
            created_at: SystemTime::UNIX_EPOCH,
            name: Default::default(),
            inventory_type: 0,
            flags: 0,
            item_type: Default::default(),
        }
    }
}

#[derive(Debug, Clone, Default)]
/// information regarding the sale of the object. How it was sold and how much it costs.
pub struct SaleInfo {
    /// The type of sale
    pub sale_type: SaleType,
    /// The price of the object
    pub price: i32,
    pub ownership_cost: Option<i32>,
}
impl SaleInfo {
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
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Missing or invalid image sale data",
            ));
        }
    }
}

#[derive(Debug, Clone, Default)]
pub enum SaleType {
    Not,
    Original,
    Copy,
    Contents,

    #[default]
    Unknown,
}
impl SaleType {
    pub fn from_i32(byte: i32) -> Self {
        match byte {
            0 => Self::Not,
            1 => Self::Original,
            2 => Self::Copy,
            3 => Self::Contents,
            _ => Self::Unknown,
        }
    }
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

#[derive(Debug, Clone, Default)]
/// Information regarding the permissions the object has
pub struct Permissions {
    pub owner_id: Uuid,
    pub group_id: Uuid,
    pub creator_id: Uuid,
    pub base_mask: i32,
    pub everyone_mask: i32,
    pub group_mask: i32,
    pub next_owner_mask: i32,
    pub owner_mask: i32,
    pub is_owner_group: Option<bool>,
    pub last_owner_id: Option<Uuid>,
}
impl Permissions {
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
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Missing or invalid image permission data",
            ));
        }
    }
}

impl Item {
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
                    sale_type: SaleType::Not,
                    price: 0,
                    ownership_cost: None,
                },
            };

            let permissions = match item.get("permissions") {
                Some(permission) => Permissions::from_llsd(permission)?,
                _ => Permissions {
                    owner_id: Uuid::nil(),
                    group_id: Uuid::nil(),
                    creator_id: Uuid::nil(),
                    base_mask: 0,
                    everyone_mask: 0,
                    group_mask: 0,
                    next_owner_mask: 0,
                    owner_mask: 0,
                    is_owner_group: None,
                    last_owner_id: None,
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
            Ok(Item {
                asset_id,
                item_id,
                created_at,
                description,
                inventory_type,
                item_type,
                flags,
                parent_id,
                name,
                permissions,
                sale_info,
            })
        } else {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Missing or invalid image data",
            ));
        }
    }
}

use std::collections::HashMap;

use uuid::Uuid;

use crate::capabilities::item_types::Permissions;

use super::item_types::{SaleInfo, SaleType};

#[derive(Debug)]
pub struct ItemData {
    pub name: String,
    pub version: String,
    pub permissions: Permissions,
    pub sale_info: SaleInfo,
    pub parameters: HashMap<i32, f32>,
    pub textures: HashMap<TextureSlot, Uuid>,
}

#[derive(Debug)]
pub struct Texture {
    texture_slot: TextureSlot,
    id: Uuid,
}

/// Could be totally wrong
#[derive(Debug, Eq, PartialEq, Hash)]
pub enum TextureSlot {
    Head = 0,
    UpperBody = 1,
    LowerBody = 2,
    Eyes = 3,
    Hair = 4,
    Shirt = 5,
    Pants = 6,
    Shoes = 7,
    Socks = 8,
    Jacket = 9,
    Unknown = 99,
}

impl TextureSlot {
    fn from_bytes(value: u8) -> Self {
        match value {
            0 => TextureSlot::Head,
            1 => TextureSlot::UpperBody,
            2 => TextureSlot::LowerBody,
            3 => TextureSlot::Eyes,
            4 => TextureSlot::Hair,
            5 => TextureSlot::Shirt,
            6 => TextureSlot::Pants,
            7 => TextureSlot::Shoes,
            8 => TextureSlot::Socks,
            9 => TextureSlot::Jacket,
            _ => TextureSlot::Unknown,
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

impl ItemData {
    pub fn from_bytes(bytes: &[u8]) -> std::io::Result<Self> {
        println!("bytes {:?}", bytes);
        let text = std::str::from_utf8(bytes)
            .map_err(|_| std::io::Error::new(std::io::ErrorKind::InvalidData, "Invalid UTF-8"))?;

        let mut lines = text.lines().map(str::trim);

        let mut next_line = || {
            lines.next().ok_or_else(|| {
                std::io::Error::new(std::io::ErrorKind::UnexpectedEof, "Unexpected end of data")
            })
        };

        let version = next_line()?.to_owned();
        let name = next_line()?.to_owned();

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

        let sale_info = SaleInfo {
            sale_type: SaleType::Not,
            price,
            ownership_cost: None,
        };

        next_line()?; // }
        next_line()?; // type 0

        let param_count = {
            let parts: Vec<&str> = next_line()?.split(' ').collect();
            parse_or_io(parts.get(1).ok_or_else(|| {
                std::io::Error::new(std::io::ErrorKind::InvalidData, "Missing parameter count")
            })?)?
        };

        let mut parameters = HashMap::new();
        for _ in 0..param_count {
            let parts: Vec<&str> = next_line()?.split(' ').collect();
            if parts.len() >= 2 {
                parameters.insert(parse_or_io(parts[0])?, parse_or_io(parts[1])?);
            }
        }

        let texture_count = {
            let parts: Vec<&str> = next_line()?.split(' ').collect();
            parse_or_io(parts.get(1).ok_or_else(|| {
                std::io::Error::new(std::io::ErrorKind::InvalidData, "Missing texture count")
            })?)?
        };

        let mut textures = HashMap::new();
        for _ in 0..texture_count {
            let parts: Vec<&str> = next_line()?.split(' ').collect();
            if parts.len() >= 2 {
                textures.insert(
                    TextureSlot::from_bytes(parse_or_io(parts[0])?),
                    parse_or_io(parts[1])?,
                );
            }
            // Optional: parse texture lines if needed
        }

        Ok(ItemData {
            version,
            name,
            permissions,
            sale_info,
            parameters,
            textures,
        })
    }
}

use super::mesh::Mesh;
use crate::utils::item_metadata::ItemMetadata;
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Debug, Default, Clone)]
/// The item struct that will be saved in the local inventory cache. If data is None, that means
/// the full data hasn't been retrieved from the asset server.
pub struct Item {
    /// Metadata about the item. Contains things like sale price, name, asset ID, description, etc.
    /// This is what is received when fetching an item from the FetchInventoryDescendents endpoint.
    pub metadata: ItemMetadata,
    /// Data about the item. Contains things like mesh information, textures, parameters, and etc.
    /// This is what is received when fetching an item's full data from the ViewerAsset endpoint.
    pub data: Option<ItemData>,
}
impl Item {
    /// Create an Item from bytes
    ///
    /// Metadata is parsed with the ItemMetadata from_bytes, and data is parsed with
    /// ItemData::from_bytes.
    pub fn from_bytes(bytes: &[u8]) -> std::io::Result<Self> {
        Ok(Item {
            metadata: ItemMetadata::from_bytes(bytes)?,
            data: Some(ItemData::from_bytes(bytes)?),
        })
    }
}

#[derive(Debug, Default, Clone)]
/// The data received from the ViewerAsset endpoint for inventory objects.
pub struct ItemData {
    /// The version of the item. For some wearables it comes in as something like "LLWearable version 22"
    pub version: String,
    /// A map of visual parameters for character customization. The key corresponds to a bodypart
    /// on the model, such as nose, or height. The value corresponds to the slider value of the
    /// modification of that bodypart. This will be rewritten to contain an enum mapping each parameter to
    /// its proper bodypart.
    pub parameters: HashMap<i32, f32>,
    /// Maps texture slots to the UUIDs of image assets.
    pub textures: HashMap<TextureSlot, Uuid>,
    /// The optional mesh attached to an object.
    /// Will not be populated until the mesh is retrieved.
    pub mesh: Option<Mesh>,
}
impl ItemData {
    /// Converts from bytes to an ItemData struct.
    /// Data for item objects come in in a newline separated bytes stream.
    /// This is not LLSD, or any other documented format, which may only be used here in ItemData.
    /// Input looks like this:
    /// ```
    /// LLWearable version 22
    /// New Eyes
    ///
    /// 	permissions 0
    /// 	{
    /// 		base_mask	7fffffff
    /// 		owner_mask	7fffffff
    ///
    /// group_mask	00000000
    /// 		everyone_mask	00000000
    /// 		next_owner_mask	00082000
    /// 		creator_id	11111111-1111-0000-0000-000100bba000
    /// 		owner_id	11111111-1111-0000-0000-000100bba000
    /// 		last_owner_id	00000000-0000-0000-0000-000000000000
    /// 		group_id	00000000-0000-0000-0000-000000000000
    /// 	}
    /// 	sale_info	0
    /// 	{
    /// 		sale_type	not
    /// 		sale_price	10
    /// 	}
    /// type 3
    /// parameters 2
    /// 98 0
    /// 99 0
    /// textures 1
    /// 3 6522e74d-1660-4e7f-b601-6f48c1659a77
    /// ```
    pub fn from_bytes(bytes: &[u8]) -> std::io::Result<Self> {
        let text = std::str::from_utf8(bytes)
            .map_err(|_| std::io::Error::new(std::io::ErrorKind::InvalidData, "Invalid UTF-8"))?;

        let mut lines = text.lines().map(str::trim);

        let mut next_line = || {
            lines.next().ok_or_else(|| {
                std::io::Error::new(std::io::ErrorKind::UnexpectedEof, "Unexpected end of data")
            })
        };

        let version = next_line()?.to_owned();
        let _name = next_line()?.to_owned();

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

        let _base_mask = parse_hex(next_line()?)?;
        let _owner_mask = parse_hex(next_line()?)?;
        let _group_mask = parse_hex(next_line()?)?;
        let _everyone_mask = parse_hex(next_line()?)?;
        let _next_owner_mask = parse_hex(next_line()?)?;

        let parse_uuid = |line: &str| {
            let id_str = line.split('\t').nth(1).ok_or_else(|| {
                std::io::Error::new(std::io::ErrorKind::InvalidData, "Missing UUID field")
            })?;
            Uuid::parse_str(id_str)
                .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string()))
        };

        let _creator_id = parse_uuid(next_line()?)?;
        let _owner_id = parse_uuid(next_line()?)?;
        let _last_owner_id = Some(parse_uuid(next_line()?)?);
        let _group_id = parse_uuid(next_line()?)?;

        next_line()?; // }
        next_line()?; // sale_info 0
        next_line()?; // {
        next_line()?; // sale_type

        let _price = {
            let _price_str = next_line()?.split('\t').nth(1).ok_or_else(|| {
                std::io::Error::new(std::io::ErrorKind::InvalidData, "Missing sale price")
            })?;
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
            parameters,
            textures,
            mesh: None,
        })
    }
}

#[derive(Debug)]
/// Texture UUID and where it gets applied
pub struct Texture {
    /// The slot where the texture is applied to, such as hair, head, eyes, etc.
    pub texture_slot: TextureSlot,
    /// The UUID of the texture in the asset server
    pub id: Uuid,
}

/// Could be totally wrong
#[derive(Debug, Eq, PartialEq, Hash, Clone)]
pub enum TextureSlot {
    /// Texture is applied to the head
    Head = 0,
    /// Texture is applied to the upper body
    UpperBody = 1,
    /// Texture is applied to the lower body
    LowerBody = 2,
    /// Texture is applied to the eyes
    Eyes = 3,
    /// Texture is applied to the hair
    Hair = 4,
    /// Texture is applied to the shirt
    Shirt = 5,
    /// Texture is applied to the pants
    Pants = 6,
    /// Texture is applied to the shoes
    Shoes = 7,
    /// Texture is applied to the socks
    Socks = 8,
    /// Texture is applied to the jacket
    Jacket = 9,
    /// Unknown
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

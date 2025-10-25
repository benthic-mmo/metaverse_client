use super::mesh::Mesh;
use crate::{errors::ParseError, utils::item_metadata::ItemMetadata};
use serde_llsd::converter::get;
use std::{
    collections::HashMap,
    str::{FromStr, from_utf8},
};
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
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, ParseError> {
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
    pub version: u32,
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
    /// ```ignore
    /// LLWearable version 22
    /// New Eyes
    ///
    ///   permissions 0
    ///   {
    ///     base_mask   7fffffff
    ///     owner_mask  7fffffff
    ///
    /// group_mask	00000000
    ///     everyone_mask	00000000
    ///     next_owner_mask	00082000
    ///     creator_id	11111111-1111-0000-0000-000100bba000
    ///     owner_id	11111111-1111-0000-0000-000100bba000
    ///     last_owner_id	00000000-0000-0000-0000-000000000000
    ///     group_id	00000000-0000-0000-0000-000000000000
    ///   }
    ///   sale_info	0
    ///   {
    ///     sale_type	not
    ///     sale_price	10
    ///   }
    /// type 3
    /// parameters 2
    /// 98 0
    /// 99 0
    /// textures 1
    /// 3 6522e74d-1660-4e7f-b601-6f48c1659a77
    /// ```
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, ParseError> {
        let data = from_utf8(bytes)?;
        let llsd =
            serde_llsd::de::auto_from_str(data).map_err(|e| ParseError::Message(e.to_string()))?;

        let map = llsd.as_map().ok_or(ParseError::LLSDError())?;
        Ok(ItemData {
            version: get("version", map),
            parameters: get("parameters", map),
            textures: get("textures", map),
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

impl FromStr for TextureSlot {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "0" => Ok(TextureSlot::Head),
            "1" => Ok(TextureSlot::UpperBody),
            "2" => Ok(TextureSlot::LowerBody),
            "3" => Ok(TextureSlot::Eyes),
            "4" => Ok(TextureSlot::Hair),
            "5" => Ok(TextureSlot::Shirt),
            "6" => Ok(TextureSlot::Pants),
            "7" => Ok(TextureSlot::Shoes),
            "8" => Ok(TextureSlot::Socks),
            "9" => Ok(TextureSlot::Jacket),
            _ => Ok(TextureSlot::Unknown),
        }
    }
}

use crate::{
    errors::ParseError,
    packet::{
        header::{Header, PacketFrequency},
        packet::{Packet, PacketData},
        packet_types::PacketType,
    },
};
use actix::Message;
use byteorder::{LittleEndian, ReadBytesExt};
use std::io::{Cursor, Read};

impl Packet {
    /// Creates a new layer data packet
    pub fn new_layer_data(layer_data: LayerData) -> Self {
        Packet {
            header: Header {
                id: 11,
                reliable: true,
                zerocoded: false,
                frequency: PacketFrequency::Low,
                ..Default::default()
            },
            body: PacketType::LayerData(Box::new(layer_data)),
        }
    }
}

#[derive(Debug, Message, Clone)]
#[rtype(result = "()")]
/// Layer data struct
pub struct LayerData {
    /// Layer type. Land, Wind, Cloud and Water. Also contains extended versions.
    pub layer_type: LayerType,
    /// Length of the data for each patch
    pub stride: u16,
    /// Size of the patch. Usually 16, representing a 16x16 grid
    pub patch_size: u8,
    /// The content of the layer, containing the patch data
    pub layer_content: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq)]
/// Enum describing types of land
pub enum LayerType {
    /// Land, a grid of compressed heightmap data
    Land,
    /// LandExtended, also a grid of compressed heightmap data
    LandExtended,
    /// TODO: document this
    Water,
    /// TODO: document this
    WaterExtended,
    /// TODO: document this
    Wind,
    /// TODO: document this
    WindExtended,
    /// TODO: document this
    Cloud,
    /// TODO: document this
    CloudExtended,
    /// Unknown layer type
    Unknown,
}

impl LayerType {
    /// Matches enum values to their bytes representation. Some of these make sense. L is 76, but
    /// all of the other values seem to be just random letters and numbers.
    pub fn to_bytes(&self) -> u8 {
        match self {
            LayerType::Land => 76,
            LayerType::LandExtended => 77,
            LayerType::Water => 87,
            LayerType::WaterExtended => 88,
            LayerType::Wind => 55,
            LayerType::WindExtended => 57,
            LayerType::Cloud => 56,
            LayerType::CloudExtended => 58,
            LayerType::Unknown => 0,
        }
    }
    /// Convert the byte representation numbers to layertype struct
    pub fn from_bytes(bytes: u8) -> Self {
        match bytes {
            76 => LayerType::Land,
            77 => LayerType::LandExtended,
            87 => LayerType::Water,
            88 => LayerType::WaterExtended,
            55 => LayerType::Wind,
            57 => LayerType::WindExtended,
            56 => LayerType::Cloud,
            58 => LayerType::CloudExtended,
            _ => LayerType::Unknown,
        }
    }
}

impl PacketData for LayerData {
    fn from_bytes(bytes: &[u8]) -> Result<Self, ParseError> {
        let mut cursor = Cursor::new(bytes);
        let layer_type_bytes = cursor.read_u8()?;
        let layer_type = LayerType::from_bytes(layer_type_bytes);

        //These bytes tell the parser how long the Data block is
        //was used to construct the size of the Data array
        //These are currently unused
        let _data_size = cursor.read_u16::<LittleEndian>()?;

        let stride = cursor.read_u16::<LittleEndian>()?;
        let patch_size = cursor.read_u8()?;

        // redundant layer type
        // could be used for validation
        let _layer_type = cursor.read_u8();

        let mut layer_content = Vec::new();
        cursor.read_to_end(&mut layer_content)?;

        let data = LayerData {
            stride,
            patch_size,
            layer_type,
            layer_content,
        };
        Ok(data)
    }
    // TOOD: fix this to_bytes function
    fn to_bytes(&self) -> Vec<u8> {
        Vec::new()
    }
}

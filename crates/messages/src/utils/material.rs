use std::fmt::{Display, Formatter, Result};

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
/// the types of materials that exist in opensimulator.
/// used for assigning textures and shaders
pub enum MaterialType {
    /// Stones and rocks
    Stone,
    /// Reflective metal
    Metal,
    /// transparent glass
    Glass,
    /// Wood
    Wood,
    /// Skin
    Flesh,
    /// Plastic
    Plastic,
    /// Rubber
    Rubber,
    /// Light. Deprecated
    Light,
    /// Undocumented
    End,
    /// Undocumented
    Mask,
    /// default unknown type
    #[default]
    Unknown,
}
impl Display for MaterialType {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        let s = match self {
            MaterialType::Stone => "stone".to_string(),
            MaterialType::Metal => "metal".to_string(),
            MaterialType::Glass => "glass".to_string(),
            MaterialType::Wood => "wood".to_string(),
            MaterialType::Flesh => "flesh".to_string(),
            MaterialType::Plastic => "plastic".to_string(),
            MaterialType::Rubber => "rubber".to_string(),
            MaterialType::Light => "light".to_string(),
            MaterialType::End => "end".to_string(),
            MaterialType::Mask => "mask".to_string(),
            MaterialType::Unknown => "unknown".to_string(),
        };
        write!(f, "{}", s)
    }
}

impl From<String> for MaterialType {
    fn from(s: String) -> Self {
        MaterialType::from(s.as_str())
    }
}
impl From<&str> for MaterialType {
    fn from(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "stone" => MaterialType::Stone,
            "metal" => MaterialType::Metal,
            "glass" => MaterialType::Glass,
            "wood" => MaterialType::Wood,
            "flesh" => MaterialType::Flesh,
            "plastic" => MaterialType::Plastic,
            "rubber" => MaterialType::Rubber,
            "light" => MaterialType::Light,
            "end" => MaterialType::End,
            "mask" => MaterialType::Mask,
            _ => MaterialType::Unknown,
        }
    }
}
impl MaterialType {
    /// Convert a MaterialType to its byte representation
    pub fn to_bytes(&self) -> u8 {
        match self {
            MaterialType::Stone => 0,
            MaterialType::Metal => 1,
            MaterialType::Glass => 2,
            MaterialType::Wood => 3,
            MaterialType::Flesh => 4,
            MaterialType::Plastic => 5,
            MaterialType::Rubber => 6,
            MaterialType::Light => 7,
            MaterialType::End => 8,
            MaterialType::Mask => 15,
            MaterialType::Unknown => 99,
        }
    }
    /// Convert a byte to its MaterialType value
    pub fn from_bytes(bytes: &u8) -> Self {
        match bytes {
            0 => MaterialType::Stone,
            1 => MaterialType::Metal,
            2 => MaterialType::Glass,
            3 => MaterialType::Wood,
            4 => MaterialType::Flesh,
            5 => MaterialType::Plastic,
            6 => MaterialType::Rubber,
            7 => MaterialType::Light,
            8 => MaterialType::End,
            15 => MaterialType::Mask,
            _ => MaterialType::Unknown,
        }
    }
}

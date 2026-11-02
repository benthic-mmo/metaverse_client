use crate::{
    udp::object::object_update::{ExtraParams, MaterialsData},
    utils::{object_types::ObjectType, path::PrimPath, texture_entry::TextureEntries},
};
use benthic_protocol::{
    avatar::types::AttachPoint, errors::ProtocolParseError, objects::types::TextureAnimation,
};
use glam::{Quat, Vec3};
use serde::{Deserialize, Serialize};
use serde_llsd_benthic::LLSDValue;
use uuid::Uuid;

/// Provides a common interface for object update packets.
/// this unifies the ObjectUpdate and ObjectUpdateCompressed packets.
pub trait ObjectUpdateData {
    /// Returns the type of the object
    fn object_type(&self) -> ObjectType;
    /// Returns the object's global id
    fn full_id(&self) -> Uuid;
    /// Returns the u32 scene-local parent ID.
    /// Objects without parents will return None
    fn parent_id(&self) -> Option<u32>;
    /// Returns the u32 scene-local ID
    fn local_id(&self) -> u32;
    /// Returns the name value string of the object.
    /// Not necessarily a name. This string can contain extra parameters used for attachments.
    fn name_value(&self) -> Option<String>;
    /// Returns the position of the object in space
    fn position(&self) -> Vec3;
    /// Returns the rotation of the object
    fn rotation(&self) -> Quat;
    /// Returns the scale of the object
    fn scale(&self) -> Vec3;
    /// Returns te texture entry data of the object.
    /// Objects without textures will return None
    fn texture(&self) -> &Option<TextureEntries>;
    /// Returns the crc of the object
    fn crc(&self) -> u32;
    /// Returns the extra parameter data of the object
    /// Objects without extra params will return None
    fn extra_params(&self) -> Option<Vec<ExtraParams>>;
    /// Returns the sculpt path of the object
    /// Objects without sculpt information will return None
    fn sculpt_path(&self) -> Option<PrimPath>;
    /// Returns the texture animation data of the object
    /// Objects without texture animation data will return None
    fn texture_animation(&self) -> Option<TextureAnimation>;

    fn pbr_materials(&self) -> Option<MaterialsData>;

    fn attachment_state(&self) -> AttachPoint;
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct PBRMaterial {
    gltf_type: String,
    version: String,
    pbr_data: PBRData,
}
impl PBRMaterial {
    pub fn from_llsd(data: LLSDValue) -> Result<PBRMaterial, ProtocolParseError> {
        if let Some(item) = data.as_map() {
            let gltf_type = match item.get("type") {
                Some(LLSDValue::String(gltf_type)) => gltf_type.to_string(),
                _ => {
                    return Err(ProtocolParseError::Message(
                        "Material has no GLTF type".to_string(),
                    ));
                }
            };
            let version = match item.get("version") {
                Some(LLSDValue::String(version)) => version.to_string(),
                _ => {
                    return Err(ProtocolParseError::Message(
                        "Material has no version".to_string(),
                    ));
                }
            };
            let pbr_data = match item.get("data") {
                Some(LLSDValue::String(json)) => {
                    serde_json::from_str::<PBRData>(json).map_err(|e| {
                        ProtocolParseError::Message(format!("Failed to deserialize PBR data: {e}"))
                    })?
                }
                _ => {
                    return Err(ProtocolParseError::Message(
                        "PBR data is not a JSON string".to_string(),
                    ));
                }
            };

            Ok(PBRMaterial {
                gltf_type,
                version,
                pbr_data,
            })
        } else {
            Err(ProtocolParseError::Message(
                "Failed to parse pbr material".to_string(),
            ))
        }
    }

    /// just retrieve the image URIs, to allow easy downloading
    pub fn image_uris(&self) -> impl Iterator<Item = &str> {
        self.pbr_data
            .images
            .iter()
            .filter_map(|image| image.uri.as_deref())
    }
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct PBRData {
    pub asset: PBRAsset,
    pub images: Vec<PBRImage>,
    pub materials: Vec<PBRMaterialDefinition>,
    pub textures: Vec<PBRTexture>,
    #[serde(rename = "extensionsUsed")]
    pub extensions_used: Option<Vec<String>>,
    #[serde(rename = "extensionsRequired")]
    pub extensions_required: Option<Vec<String>>,
    pub extras: Option<serde_json::Value>,
    pub samplers: Option<Vec<serde_json::Value>>,
    pub scene: Option<usize>,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct PBRAsset {
    pub version: String,
    #[serde(rename = "minVersion")]
    pub min_version: Option<String>,
    pub generator: Option<String>,
    pub copyright: Option<String>,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct PBRImage {
    pub uri: Option<String>,
    pub name: Option<String>,
    #[serde(rename = "mimeType")]
    pub mime_type: Option<String>,
    #[serde(rename = "bufferView")]
    pub buffer_view: Option<usize>,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct PBRTexture {
    pub source: usize,
    pub sampler: Option<usize>,
    pub name: Option<String>,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct PBRMaterialDefinition {
    #[serde(rename = "emissiveFactor")]
    pub emissive_factor: Option<[f32; 3]>,

    #[serde(rename = "emissiveTexture")]
    pub emissive_texture: Option<PBRTextureInfo>,

    #[serde(rename = "normalTexture")]
    pub normal_texture: Option<PBRTextureInfo>,

    #[serde(rename = "occlusionTexture")]
    pub occlusion_texture: Option<PBRTextureInfo>,

    #[serde(rename = "pbrMetallicRoughness")]
    pub pbr_metallic_roughness: Option<PBRMetallicRoughness>,

    #[serde(rename = "alphaMode")]
    pub alpha_mode: Option<AlphaMode>,

    #[serde(rename = "alphaCutoff")]
    pub alpha_cutoff: Option<f32>,

    #[serde(rename = "doubleSided")]
    pub double_sided: Option<bool>,

    pub name: Option<String>,

    pub extras: Option<PBRExtras>,
    //
    pub extensions: Option<serde_json::Value>,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct PBRExtras {
    pub override_alpha_mode: Option<bool>,
    pub override_double_sided: Option<bool>,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct PBRTextureInfo {
    pub index: usize,
    #[serde(rename = "texCoord")]
    pub tex_coord: Option<usize>, // default 0
    pub extensions: Option<serde_json::Value>, // KHR_texture_transform
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct PBRMetallicRoughness {
    #[serde(rename = "baseColorTexture")]
    pub base_color_texture: Option<PBRTextureInfo>,

    #[serde(rename = "metallicRoughnessTexture")]
    pub metallic_roughness_texture: Option<PBRTextureInfo>,

    #[serde(rename = "baseColorFactor")]
    pub base_color_factor: Option<[f32; 4]>,

    #[serde(rename = "metallicFactor")]
    pub metallic_factor: Option<f32>,

    #[serde(rename = "roughnessFactor")]
    pub roughness_factor: Option<f32>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum AlphaMode {
    #[serde(rename = "OPAQUE")]
    Opaque,
    #[serde(rename = "MASK")]
    Mask,
    #[serde(rename = "BLEND")]
    Blend,
}
impl<'de> Deserialize<'de> for AlphaMode {
    fn deserialize<D>(d: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        Ok(match String::deserialize(d)?.as_str() {
            "MASK" => AlphaMode::Mask,
            "BLEND" => AlphaMode::Blend,
            _ => AlphaMode::Opaque,
        })
    }
}

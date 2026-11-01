use crate::avatar::types::{AttachmentData, JointWeight, SkinData};
use glam::{Quat, Vec3};
use rgb::Rgba;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use uuid::Uuid;

/// Trait for messages that are retriable.
pub trait RetriableData {
    /// How many times this object has been retried
    fn retry_count(&self) -> u32;
    /// A function to incrememnt the retry count for an object
    fn inc_retry_count(&mut self);
}

/// Common values for object instances
#[derive(Debug, Clone)]
pub struct ObjectInstanceInfo {
    /// full, globally unique UUID of the object's instance
    pub full_id: Uuid,
    /// scene-local u32 ID of the object's instance
    pub local_id: u32,
    /// scene-local u32 ID of the object's parent
    pub parent_id: Option<u32>,
    /// Position in space of the object
    pub position: Vec3,
    /// Scale of the object
    /// This scale is NOT relative to the parent. This is the global scale of the object.
    pub scale: Vec3,
    /// Rotation of the object
    pub rotation: Quat,
}

///TODO: unimplemeted
pub struct AttachmentObjectData {
    /// ID of the object this is attached to
    pub parent_id: Option<u32>,
}

/// TODO: unimplemented
pub struct TreeObjectData {}

/// TODO: unimplemeted
pub struct GrassObjectData {}

/// TODO: unimplemented
pub struct UnknownObject {}

/// TODO: unimplemented
pub struct ParticleObjectData {}

/// TODO: unimplemented
pub struct NewTreeObjectData {}
/// Face for render objects
/// This is treated as its own sub-mesh when building mesh objects.
/// can contain its own vertices, indices, normals, weights and textures.
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct RenderFace {
    /// index of the face
    pub face_index: u32,
    /// vertices in the face
    pub vertices: Vec<Vec3>,
    /// indices for the vertices in the fae
    pub indices: Vec<u32>,
    /// UV values for vertices in the face
    pub uv: Vec<[f32; 2]>,
    /// Normals for the UVs in the face
    pub normals: Option<Vec<Vec3>>,
    /// Weights of the vertices in the face
    pub weights: Option<Vec<JointWeight>>,
    /// Material and texture used by the face
    pub material: Option<MaterialModel>,
}

/// Type of material used by the mesh
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MaterialModel {
    /// Material is using blinn-phong data.
    /// Legacy to be ignored in favor of PBR materials, if they exist.
    BlinnPhong(BlinnPhongMaterial),
    /// Material is using PBR data.
    /// If PBR data is present, this is always used
    PBR(PBRMaterial),
}

/// Material information using the Blinn-Phong material model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlinnPhongMaterial {
    /// Path to the texture
    pub texture_path: PathBuf,
    /// Color tint of the texture
    pub color: Rgba<u8>,
    /// Transform applied to the texture
    /// this is face-specific, and effectively moves, repeats, rotates and offests a texture.
    pub texture_transform: TextureTransform,
    /// Animations applied to the texture.
    /// Textures can animate using rotations, slides, or jumps between frames baked into a texture.
    pub texture_animation: Option<TextureAnimation>,
    /// Level of reflectivity of the texture
    pub shiny: ShinyLevel,
    /// Bool to determine if this texture is affected by lighting
    pub fullbright: bool,
    /// value to determine how brightly this material glows
    pub glow: f32,
    /// Procedurally generated bump map, used to give visual texture to an untextured mesh.
    pub bump: BumpMap,
    /// Information on what kind of projection to use for the texture's UV coordinates
    pub tex_gen: TexGen,
}

/// Transform data for adjusting a texture's rendered location.
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct TextureTransform {
    /// Horizontal texture repetition factor
    pub repeat_u: f32,
    /// Vertical texture repitition factor
    pub repeat_v: f32,
    /// Horizontal texture offset
    pub offset_u: f32,
    /// Vertical texture offset
    pub offset_v: f32,
    /// Texture rotation angle in radians
    pub rotation: f32,
}

/// TODO: unimplemented
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct PBRMaterial {
    /// path on disk to the texture
    pub texture_path: PathBuf,
}

/// This contains required data that will be used for rendering.
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct RenderObject {
    /// Name of the object
    pub name: String,
    /// globally unique ID of the object
    pub id: Uuid,
    /// render object face information
    pub faces: Vec<RenderFace>,
    /// Optional data describing where this object is attached
    pub attachment_data: Option<AttachmentData>,
    /// Otional skinning data
    pub skin: Option<SkinData>,
}

/// Bitmask used to extract the texture mapping mode from face flags.
pub const TEX_GEN_MASK: u8 = 0x06;

/// Specifies how texture coordinates are generated for a surface.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TexGen {
    /// Uses the default texture coordinate mapping.
    Default,
    /// Generates texture coordinates using planar projection.
    Planar,
    /// Generates texture coordinates using spherical projection.
    Spherical,
    /// Generates texture coordinates using cylindrical projection.
    Cylindrical,
}
impl TexGen {
    /// Convert bits to a tex gen mask
    pub fn from_bits(bits: u8) -> Self {
        match bits & TEX_GEN_MASK {
            0x02 => TexGen::Planar,
            0x04 => TexGen::Spherical,
            0x06 => TexGen::Cylindrical,
            _ => TexGen::Default,
        }
    }
}

/// Level of shininess of a texture
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ShinyLevel {
    /// Not shiny
    None,
    /// Not very shiny
    Low,
    /// Pretty shiny
    Medium,
    /// Very shiny
    High,
}
impl ShinyLevel {
    /// Convert bits to a shiny level
    pub fn from_bits(bits: u8) -> Self {
        match bits {
            0 => ShinyLevel::None,
            1 => ShinyLevel::Low,
            2 => ShinyLevel::Medium,
            _ => ShinyLevel::High,
        }
    }
}

/// Bitmask used to extract the bump mapping type from face flags.
pub const BUMP_MASK: u8 = 0x1f;
/// Specifies the bump mapping pattern applied to a surface.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BumpMap {
    /// Disables bump mapping.
    None,
    /// Uses brightness variations to generate bump details.
    Brightness,
    /// Uses darkness variations to generate bump details.
    Darkness,
    /// Uses a woodgrain bump pattern.
    Woodgrain,
    /// Uses a bark bump pattern.
    Bark,
    /// Uses a brick bump pattern.
    Bricks,
    /// Uses a checkerboard bump pattern.
    Checker,
    /// Uses a concrete bump pattern.
    Concrete,
    /// Uses a tile bump pattern.
    Tile,
    /// Uses a stone bump pattern.
    Stone,
    /// Uses a disc-shaped bump pattern.
    Discs,
    /// Uses a gravel bump pattern.
    Gravel,
    /// Uses a blob-shaped bump pattern.
    Blobs,
    /// Uses a siding bump pattern.
    Siding,
    /// Uses a large-tile bump pattern.
    Largetile,
    /// Uses a stucco bump pattern.
    Stucco,
    /// Uses a suction-pattern bump map.
    Suction,
    /// Uses a woven bump pattern.
    Weave,
    /// Uses an unsupported bump pattern
    Other(u8),
}
impl BumpMap {
    /// convert from bits to a bump map enum
    pub fn from_bits(bits: u8) -> Self {
        match bits & BUMP_MASK {
            0 => Self::None,
            1 => Self::Brightness,
            2 => Self::Darkness,
            3 => Self::Woodgrain,
            4 => Self::Bark,
            5 => Self::Bricks,
            6 => Self::Checker,
            7 => Self::Concrete,
            8 => Self::Tile,
            9 => Self::Stone,
            10 => Self::Discs,
            11 => Self::Gravel,
            12 => Self::Blobs,
            13 => Self::Siding,
            14 => Self::Largetile,
            15 => Self::Stucco,
            16 => Self::Suction,
            17 => Self::Weave,
            v => Self::Other(v),
        }
    }
    /// convert from a bump map pattern to bits
    pub fn to_bits(self) -> u8 {
        match self {
            Self::None => 0,
            Self::Brightness => 1,
            Self::Darkness => 2,
            Self::Woodgrain => 3,
            Self::Bark => 4,
            Self::Bricks => 5,
            Self::Checker => 6,
            Self::Concrete => 7,
            Self::Tile => 8,
            Self::Stone => 9,
            Self::Discs => 10,
            Self::Gravel => 11,
            Self::Blobs => 12,
            Self::Siding => 13,
            Self::Largetile => 14,
            Self::Stucco => 15,
            Self::Suction => 16,
            Self::Weave => 17,
            Self::Other(v) => v & BUMP_MASK,
        }
    }
}

/// Flag indicating that the animation is enabled.
pub const MODE_ON: u8 = 0x01;
/// Flag indicating that the animation uses smooth transitions.
pub const MODE_SMOOTH: u8 = 0x02;
/// Flag indicating that the texture rotates during animation.
pub const MODE_ROTATE: u8 = 0x04;
/// Flag indicating that the texture scales during animation.
pub const MODE_SCALE: u8 = 0x08;
/// Flag indicating that the animation runs in reverse.
pub const MODE_REVERSE: u8 = 0x10;
/// Flag indicating that the animation reverses direction at each endpoint.
pub const MODE_PING_PONG: u8 = 0x20;

/// Enum to determine if a texture animation should play
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AnimationState {
    /// Animation is off
    Off,
    /// Animation is on
    On,
}

/// Enum to describe how to play the texture animation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AnimationStyle {
    /// The animation is frame-by-frame, containing animation frames baked into the texture.
    /// move the view to the next frame once the current one is complete.
    Frame,
    /// Move the view smoothly over the texture.
    Smooth,
}

/// Enum to describe how to rotate the texture animation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RotationMode {
    /// Animation does not rotate
    None,
    /// Animation does rotate
    Rotate,
}

/// Enum to describe how to scale a texture animation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ScaleMode {
    /// Texture does not scale
    None,
    /// Texture does scale
    Scale,
}

/// Enum to describe in which direction a texture animation plays
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PlaybackDirection {
    /// The animation plays forward
    Forward,
    /// The animation plays reversed
    Reverse,
}

/// Enum to describe if an animation loops or ping-pongs
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PlaybackMode {
    /// Animation loops
    Loop,
    /// Animation plays forward, then reversed.
    PingPong,
}

/// Describes a texture animation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextureAnimation {
    /// If the animation is turned on
    pub state: AnimationState,
    /// Describes how the animation should be played
    pub style: AnimationStyle,
    /// Describes if the animation rotates
    pub rotation: RotationMode,
    /// Describes if the animation scales
    pub scale: ScaleMode,
    /// Describes which direction the animation plays
    pub direction: PlaybackDirection,
    /// Describes if the animation loops, or plays forward and then plays reversed.
    pub playback: PlaybackMode,
    /// Face the animation applies to
    pub face: i8,
    /// Size of the animation's frame
    /// creates a square starting at x
    pub size_x: u8,
    /// Size of the animation's frame
    /// Creates a square ending at Y
    pub size_y: u8,
    /// Start position of the animation
    pub start: f32,
    /// How many frames are in the animation
    pub length: f32,
    /// how quickly to play the animation
    pub rate: f32,
}
impl TextureAnimation {
    /// convert bytes to a TextureAnimation object
    /// Requires 16 bytes exactly, or else this will return None.
    pub fn from_bytes(bytes: &[u8]) -> Option<Self> {
        if bytes.len() != 16 {
            return None;
        }

        let mode = bytes[0];

        if mode & MODE_ON == 0 {
            return None;
        }

        let state = AnimationState::On;

        let style = if mode & MODE_SMOOTH != 0 {
            AnimationStyle::Smooth
        } else {
            AnimationStyle::Frame
        };

        let rotation = if mode & MODE_ROTATE != 0 {
            RotationMode::Rotate
        } else {
            RotationMode::None
        };

        let scale = if mode & MODE_SCALE != 0 {
            ScaleMode::Scale
        } else {
            ScaleMode::None
        };

        let direction = if mode & MODE_REVERSE != 0 {
            PlaybackDirection::Reverse
        } else {
            PlaybackDirection::Forward
        };

        let playback = if mode & MODE_PING_PONG != 0 {
            PlaybackMode::PingPong
        } else {
            PlaybackMode::Loop
        };

        let mut size_x = bytes[2];
        let mut size_y = bytes[3];

        if style != AnimationStyle::Smooth {
            size_x = size_x.max(1);
            size_y = size_y.max(1);
        }

        Some(Self {
            state,
            style,
            rotation,
            scale,
            direction,
            playback,
            face: bytes[1] as i8,
            size_x,
            size_y,
            start: f32::from_le_bytes(bytes[4..8].try_into().unwrap()),
            length: f32::from_le_bytes(bytes[8..12].try_into().unwrap()),
            rate: f32::from_le_bytes(bytes[12..16].try_into().unwrap()),
        })
    }
}

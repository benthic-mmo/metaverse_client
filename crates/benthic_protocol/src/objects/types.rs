use crate::avatar::types::{AttachmentData, JointWeight, SkinData};
use glam::Vec3;
use rgb::Rgba;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use uuid::Uuid;

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct RenderFace {
    pub face_index: u32,
    pub vertices: Vec<Vec3>,
    pub indices: Vec<u32>,
    pub uv: Vec<[f32; 2]>,
    pub normals: Option<Vec<Vec3>>,
    pub weights: Option<Vec<JointWeight>>,
    pub material: Option<MaterialModel>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MaterialModel {
    BlinnPhong(BlinnPhongMaterial),
    PBR(PBRMaterial),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlinnPhongMaterial {
    pub texture_path: PathBuf,
    pub color: Rgba<u8>,
    pub texture_transform: TextureTransform,
    pub texture_animation: Option<TextureAnimation>,
    pub shiny: ShinyLevel,
    pub fullbright: bool,
    pub glow: f32,
    pub bump: BumpMap,
    pub tex_gen: TexGen,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct TextureTransform {
    pub repeat_u: f32,
    pub repeat_v: f32,
    pub offset_u: f32,
    pub offset_v: f32,
    pub rotation: f32,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct PBRMaterial {
    pub texture_path: PathBuf,
}

/// This contains required data that will be used for rendering.
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct RenderObject {
    pub name: String,
    pub id: Uuid,
    pub faces: Vec<RenderFace>,
    pub attachment_data: Option<AttachmentData>,
    pub skin: Option<SkinData>,
}

pub const TEX_GEN_MASK: u8 = 0x06;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TexGen {
    Default,
    Planar,
    Spherical,
    Cylindrical,
}
impl TexGen {
    pub fn from_bits(bits: u8) -> Self {
        match bits & TEX_GEN_MASK {
            0x02 => TexGen::Planar,
            0x04 => TexGen::Spherical,
            0x06 => TexGen::Cylindrical,
            _ => TexGen::Default,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ShinyLevel {
    None,
    Low,
    Medium,
    High,
}

impl ShinyLevel {
    pub fn from_bits(bits: u8) -> Self {
        match bits {
            0 => ShinyLevel::None,
            1 => ShinyLevel::Low,
            2 => ShinyLevel::Medium,
            _ => ShinyLevel::High,
        }
    }
}

pub const BUMP_MASK: u8 = 0x1f;
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BumpMap {
    None,
    Brightness,
    Darkness,
    Woodgrain,
    Bark,
    Bricks,
    Checker,
    Concrete,
    Tile,
    Stone,
    Discs,
    Gravel,
    Blobs,
    Siding,
    Largetile,
    Stucco,
    Suction,
    Weave,
    Other(u8),
}
impl BumpMap {
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

pub const MODE_ON: u8 = 0x01;
pub const MODE_SMOOTH: u8 = 0x02;
pub const MODE_ROTATE: u8 = 0x04;
pub const MODE_SCALE: u8 = 0x08;
pub const MODE_REVERSE: u8 = 0x10;
pub const MODE_PING_PONG: u8 = 0x20;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AnimationState {
    Off,
    On,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AnimationStyle {
    Frame,
    Smooth,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RotationMode {
    None,
    Rotate,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ScaleMode {
    None,
    Scale,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PlaybackDirection {
    Forward,
    Reverse,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PlaybackMode {
    Loop,
    PingPong,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextureAnimation {
    pub state: AnimationState,
    pub style: AnimationStyle,
    pub rotation: RotationMode,
    pub scale: ScaleMode,
    pub direction: PlaybackDirection,
    pub playback: PlaybackMode,
    pub face: i8,
    pub size_x: u8,
    pub size_y: u8,
    pub start: f32,
    pub length: f32,
    pub rate: f32,
}

impl TextureAnimation {
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

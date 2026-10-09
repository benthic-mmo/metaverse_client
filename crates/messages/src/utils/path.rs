use crate::{errors::ParseError, parse};
use byteorder::{LittleEndian, ReadBytesExt};
use serde::{Deserialize, Serialize};
use std::{
    hash::{DefaultHasher, Hash, Hasher},
    io::Cursor,
};

const CUT_QUANTA: f32 = 0.00002;
const SCALE_QUANTA: f32 = 0.01;
const SHEAR_QUANTA: f32 = 0.01;
const TAPER_QUANTA: f32 = 0.01;
const REV_QUANTA: f32 = 0.015;
const HOLLOW_QUANTA: f32 = 0.00002;

/// This contains path information. This contains information about how a basic shape
/// can be stretched, tapered, twisted, sheared and deformed.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PrimPath {
    /// This determines the type of path the shape follows.
    /// if it is a straight line, a circle, or etc.
    pub curve: PathCurve,

    /// Path cut start, values between 0.0 and 1.0
    pub begin: f32,

    /// Path cut end, values between 0.0 and 1.0
    pub end: f32,

    /// Scale of the profile at the end of the path on X, values between 0 and 200
    pub scale_x: f32,

    /// Scale of the profile at the end of the path on Y, values between 0 and 200
    pub scale_y: f32,

    /// Top shear on X, between -127 and 127
    pub shear_x: f32,

    /// Top shear on Y, between -127 and 127
    pub shear_y: f32,

    /// Twist applied at the path end
    pub twist_end: f32,

    /// Twist applied at the path start
    pub twist_begin: f32,

    /// Radius offset — how far the path spirals away from the axis
    pub radius_offset: f32,

    /// Taper on X
    pub taper_x: f32,

    /// Taper on Y
    pub taper_y: f32,

    /// Revolutions around the path
    pub revolutions: f32,

    /// Skew along the path
    /// 0 = no skew.
    pub skew: f32,

    /// Profile cut start, values between 0.0 and 1.0
    pub profile_begin: f32,

    /// Profile cut end, values between 0.0 and 1.0
    pub profile_end: f32,

    /// Hollow size, between 0.0 and 1.0
    /// A hollow cylinder becomes a tube
    pub profile_hollow: f32,

    /// Hole shape
    pub hollow_shape: HollowShape,

    /// Profile shape
    pub profile_shape: ProfileShape,
}
impl PrimPath {
    pub fn hash(&self) -> u64 {
        let mut hasher = DefaultHasher::new();

        self.curve.hash(&mut hasher);
        self.begin.to_bits().hash(&mut hasher);
        self.end.to_bits().hash(&mut hasher);
        self.scale_x.to_bits().hash(&mut hasher);
        self.scale_y.to_bits().hash(&mut hasher);
        self.shear_x.to_bits().hash(&mut hasher);
        self.shear_y.to_bits().hash(&mut hasher);
        self.twist_end.to_bits().hash(&mut hasher);
        self.twist_begin.to_bits().hash(&mut hasher);
        self.radius_offset.to_bits().hash(&mut hasher);
        self.taper_x.to_bits().hash(&mut hasher);
        self.taper_y.to_bits().hash(&mut hasher);
        self.revolutions.to_bits().hash(&mut hasher);
        self.skew.to_bits().hash(&mut hasher);
        self.profile_begin.to_bits().hash(&mut hasher);
        self.profile_end.to_bits().hash(&mut hasher);
        self.profile_hollow.to_bits().hash(&mut hasher);

        self.hollow_shape.hash(&mut hasher);
        self.profile_shape.hash(&mut hasher);

        hasher.finish()
    }
}

/// How the 2D profile cross-section is swept through space to form the prim.
/// Sent as the PathCurve byte in the object update shape block.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u8)]
pub enum PathCurve {
    /// Extrude the profile in a straight line → box, cylinder, prism.
    #[default]
    Line = 16,
    /// Sweep the profile around a circular path → torus, sphere, tube, ring.
    Circle = 32,
    /// Alternate circular sweep → sphere.
    Circle2 = 48,
    /// Test/legacy value, unused in practice.
    Test = 64,
    /// Linear extrusion + flexible prim (always paired with the Flexible extra param).
    Flexible = 128,
}

impl PathCurve {
    pub fn from_byte(byte: u8) -> Self {
        match byte {
            16 => Self::Line,
            32 => Self::Circle,
            48 => Self::Circle2,
            64 => Self::Test,
            128 => Self::Flexible,
            _ => Self::Line,
        }
    }

    pub fn from_string(value: &str) -> Option<Self> {
        match value {
            "Line" => Some(Self::Line),
            "Circle" => Some(Self::Circle),
            "Circle2" => Some(Self::Circle2),
            "Test" => Some(Self::Test),
            "Flexible" => Some(Self::Flexible),
            _ => None,
        }
    }

    pub fn hash(&self, hasher: &mut impl Hasher) {
        (*self as u8).hash(hasher);
    }

    pub fn high_resolution_steps(self) -> f32 {
        24.0
    }
    pub fn medium_resolution_steps(self) -> f32 {
        12.0
    }
    pub fn low_resolution_steps(self) -> f32 {
        6.0
    }
    /// Whether this path extrudes linearly rather than sweeping in a circle.
    pub fn is_linear(self) -> bool {
        matches!(self, Self::Line | Self::Flexible)
    }
}

/// Shape of the 2D profile cross-section (low nibble of the packed byte).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u8)]
pub enum ProfileShape {
    #[default]
    Circle = 0,
    Square = 1,
    IsoTriangle = 2,
    EqualTriangle = 3,
    RightTriangle = 4,
    HalfCircle = 5,
    Unknown = 255,
}
impl ProfileShape {
    pub fn hash(&self, hasher: &mut impl Hasher) {
        (*self as u8).hash(hasher);
    }

    pub fn from_string(value: &str) -> Option<Self> {
        match value {
            "Circle" => Some(Self::Circle),
            "Square" => Some(Self::Square),
            "IsoTriangle" => Some(Self::IsoTriangle),
            "EqualTriangle" => Some(Self::EqualTriangle),
            "RightTriangle" => Some(Self::RightTriangle),
            "HalfCircle" => Some(Self::HalfCircle),
            "Unknown" => Some(Self::Unknown),
            _ => None,
        }
    }
    pub fn default_sides_high(self) -> usize {
        match self {
            Self::Circle => 24,
            Self::HalfCircle => 24,
            Self::Square => 4,
            Self::IsoTriangle | Self::RightTriangle => 4,
            Self::EqualTriangle => 3,
            Self::Unknown => 4,
        }
    }
    pub fn default_sides_medium(self) -> usize {
        match self {
            Self::Circle => 12,
            Self::HalfCircle => 12,
            Self::Square => 4,
            Self::IsoTriangle | Self::RightTriangle => 4,
            Self::EqualTriangle => 3,
            Self::Unknown => 4,
        }
    }
    pub fn default_sides_low(self) -> usize {
        match self {
            Self::Circle => 6,
            Self::HalfCircle => 6,
            Self::Square => 4,
            Self::IsoTriangle | Self::RightTriangle => 4,
            Self::EqualTriangle => 3,
            Self::Unknown => 4,
        }
    }
    fn from_nibble(nibble: u8) -> Self {
        match nibble {
            0 => Self::Circle,
            1 => Self::Square,
            2 => Self::IsoTriangle,
            3 => Self::EqualTriangle,
            4 => Self::RightTriangle,
            5 => Self::HalfCircle,
            _ => Self::Unknown,
        }
    }
}
/// Shape of the hollow cut into the profile (high nibble of the packed byte).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u8)]
pub enum HollowShape {
    #[default]
    Same = 0,
    Circle = 1,
    Square = 2,
    Triangle = 3,
    Unknown = 255,
}

impl HollowShape {
    pub fn hash(&self, hasher: &mut impl Hasher) {
        (*self as u8).hash(hasher);
    }
    pub fn from_string(value: &str) -> Option<Self> {
        match value {
            "Same" => Some(Self::Same),
            "Circle" => Some(Self::Circle),
            "Square" => Some(Self::Square),
            "Triangle" => Some(Self::Triangle),
            "Unknown" => Some(Self::Unknown),
            _ => None,
        }
    }

    fn from_nibble(nibble: u8) -> Self {
        match nibble {
            0 => Self::Same,
            1 => Self::Circle,
            2 => Self::Square,
            3 => Self::Triangle,
            _ => Self::Unknown,
        }
    }
}

impl PrimPath {
    /// Decodes the 26-byte shape block from an ObjectUpdate/ObjectAdd/ObjectShape packet.
    /// Values are kept in their raw encoded types without scaling or casting.
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, ParseError> {
        let mut cursor = Cursor::new(bytes);

        let curve = PathCurve::from_byte(parse!(cursor.read_u8())?);
        let profile_curve = parse!(cursor.read_u8())?;

        let profile_shape = ProfileShape::from_nibble(profile_curve & 0x0F);
        let hollow_shape = HollowShape::from_nibble(profile_curve >> 4);

        let begin = parse!(cursor.read_u16::<LittleEndian>())? as f32 * CUT_QUANTA;
        let end = (50000 - parse!(cursor.read_u16::<LittleEndian>())? as i32) as f32 * CUT_QUANTA;
        let scale_x = (200 - parse!(cursor.read_u8())? as i32) as f32 * SCALE_QUANTA;
        let scale_y = (200 - parse!(cursor.read_u8())? as i32) as f32 * SCALE_QUANTA;
        let shear_x = parse!(cursor.read_i8())? as f32 * SHEAR_QUANTA;
        let shear_y = parse!(cursor.read_i8())? as f32 * SHEAR_QUANTA;
        let twist_end = parse!(cursor.read_i8())? as f32 * SCALE_QUANTA;
        let twist_begin = parse!(cursor.read_i8())? as f32 * SCALE_QUANTA;
        let radius_offset = parse!(cursor.read_i8())? as f32 * SCALE_QUANTA;
        let taper_x = parse!(cursor.read_i8())? as f32 * TAPER_QUANTA;
        let taper_y = parse!(cursor.read_i8())? as f32 * TAPER_QUANTA;
        let revolutions = parse!(cursor.read_u8())? as f32 * REV_QUANTA + 1.0;
        let skew = parse!(cursor.read_i8())? as f32 * SCALE_QUANTA;
        let profile_begin = parse!(cursor.read_u16::<LittleEndian>())? as f32 * CUT_QUANTA;
        let profile_end =
            (50000 - parse!(cursor.read_u16::<LittleEndian>())? as i32) as f32 * CUT_QUANTA;
        let profile_hollow = parse!(cursor.read_u16::<LittleEndian>())? as f32 * HOLLOW_QUANTA;

        Ok(Self {
            curve,
            begin,
            end,
            scale_x,
            scale_y,
            shear_x,
            shear_y,
            twist_end,
            twist_begin,
            radius_offset,
            taper_x,
            taper_y,
            revolutions,
            skew,
            profile_begin,
            profile_end,
            profile_hollow,
            hollow_shape,
            profile_shape,
        })
    }
    /// for some reason the compressed packet has a slightly different order.
    /// why.
    pub fn from_bytes_compressed(bytes: &[u8]) -> Result<Self, ParseError> {
        let mut cursor = Cursor::new(bytes);

        let curve = PathCurve::from_byte(parse!(cursor.read_u8())?);
        let begin = parse!(cursor.read_u16::<LittleEndian>())? as f32 * CUT_QUANTA;
        let end = (50000 - parse!(cursor.read_u16::<LittleEndian>())? as i32) as f32 * CUT_QUANTA;
        let scale_x = (200 - parse!(cursor.read_u8())? as i32) as f32 * SCALE_QUANTA;
        let scale_y = (200 - parse!(cursor.read_u8())? as i32) as f32 * SCALE_QUANTA;
        let shear_x = parse!(cursor.read_i8())? as f32 * SHEAR_QUANTA;
        let shear_y = parse!(cursor.read_i8())? as f32 * SHEAR_QUANTA;
        let twist_end = parse!(cursor.read_i8())? as f32 * SCALE_QUANTA;
        let twist_begin = parse!(cursor.read_i8())? as f32 * SCALE_QUANTA;
        let radius_offset = parse!(cursor.read_i8())? as f32 * SCALE_QUANTA;
        let taper_x = parse!(cursor.read_i8())? as f32 * TAPER_QUANTA;
        let taper_y = parse!(cursor.read_i8())? as f32 * TAPER_QUANTA;
        let revolutions = parse!(cursor.read_u8())? as f32 * REV_QUANTA + 1.0;
        let skew = parse!(cursor.read_i8())? as f32 * SCALE_QUANTA;
        let profile_curve = parse!(cursor.read_u8())?;

        let profile_shape = ProfileShape::from_nibble(profile_curve & 0x0F);
        let hollow_shape = HollowShape::from_nibble(profile_curve & 0xF0);

        let profile_begin = parse!(cursor.read_u16::<LittleEndian>())? as f32 * CUT_QUANTA;
        let profile_end =
            (50000 - parse!(cursor.read_u16::<LittleEndian>())? as i32) as f32 * CUT_QUANTA;
        let profile_hollow = parse!(cursor.read_u16::<LittleEndian>())? as f32 * HOLLOW_QUANTA;

        Ok(Self {
            curve,
            begin,
            end,
            scale_x,
            scale_y,
            shear_x,
            shear_y,
            twist_end,
            twist_begin,
            radius_offset,
            taper_x,
            taper_y,
            revolutions,
            skew,
            profile_begin,
            profile_end,
            profile_hollow,
            hollow_shape,
            profile_shape,
        })
    }
}

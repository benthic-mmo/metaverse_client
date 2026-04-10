use crate::errors::ParseError;
use glam::{Quat, Vec2, Vec3};
use rgb::Rgb;
use serde::{Deserialize, Serialize};
use serde_llsd_benthic::{de::xml, LLSDValue};
use std::{collections::HashMap, str::FromStr};
use uuid::Uuid;

macro_rules! llsd_get {
    ($map:expr, $key:expr, $method:ident) => {{
        let v = $map
            .get($key)
            .ok_or_else(|| ParseError::MissingField($key.into()))?;

        v.$method().ok_or_else(|| {
            ParseError::InvalidField(format!(
                "Invalid field {}: {:?}, cannot convert using: {}",
                $key,
                v,
                stringify!($method)
            ))
        })
    }};
}

macro_rules! llsd_rgb {
    ($map:expr, $key:expr) => {{
        let arr = llsd_get!($map, $key, as_array)?;

        let mut iter = arr.iter();

        let r =
            *iter.next().and_then(|v| v.as_real()).ok_or_else(|| {
                ParseError::InvalidField(format!("Incorrect Red Value in {}", $key))
            })? as f32;

        let g =
            *iter.next().and_then(|v| v.as_real()).ok_or_else(|| {
                ParseError::InvalidField(format!("Incorrect Green Value in {}", $key))
            })? as f32;

        let b =
            *iter.next().and_then(|v| v.as_real()).ok_or_else(|| {
                ParseError::InvalidField(format!("Incorrect Blue Value in {}", $key))
            })? as f32;

        Rgb { r, g, b }
    }};
}

macro_rules! llsd_vec3 {
    ($map:expr, $key:expr) => {{
        let arr = llsd_get!($map, $key, as_array)?;
        let mut iter = arr.iter();

        let x = *iter
            .next()
            .and_then(|v| v.as_real())
            .ok_or_else(|| ParseError::InvalidField("bad x".into()))? as f32;

        let y = *iter
            .next()
            .and_then(|v| v.as_real())
            .ok_or_else(|| ParseError::InvalidField("bad y".into()))? as f32;

        let z = *iter
            .next()
            .and_then(|v| v.as_real())
            .ok_or_else(|| ParseError::InvalidField("bad z".into()))? as f32;

        glam::Vec3::new(x, y, z)
    }};
}

macro_rules! llsd_vec2 {
    ($map:expr, $key:expr) => {{
        let arr = llsd_get!($map, $key, as_array)?;
        let mut iter = arr.iter();

        let x = *iter
            .next()
            .and_then(|v| v.as_real())
            .ok_or_else(|| ParseError::InvalidField("bad x".into()))? as f32;

        let y = *iter
            .next()
            .and_then(|v| v.as_real())
            .ok_or_else(|| ParseError::InvalidField("bad y".into()))? as f32;

        glam::Vec2::new(x, y)
    }};
}

macro_rules! llsd_quat {
    ($map:expr, $key:expr) => {{
        let arr = llsd_get!($map, $key, as_array)?;
        let mut iter = arr.iter();

        let x = *iter
            .next()
            .and_then(|v| v.as_real())
            .ok_or_else(|| ParseError::InvalidField("bad quat x".into()))? as f32;

        let y = *iter
            .next()
            .and_then(|v| v.as_real())
            .ok_or_else(|| ParseError::InvalidField("bad quat y".into()))? as f32;

        let z = *iter
            .next()
            .and_then(|v| v.as_real())
            .ok_or_else(|| ParseError::InvalidField("bad quat z".into()))? as f32;

        let w = *iter
            .next()
            .and_then(|v| v.as_real())
            .ok_or_else(|| ParseError::InvalidField("bad quat w".into()))? as f32;

        glam::Quat::from_xyzw(x, y, z, w)
    }};
}

/// A DayCycle as defined by the Environmental Enhancement Project.
/// this contains information about the length of a day, and the keyframes the sky will go through
/// as the day progresses. This creates things like sunsets, and celestial events.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DayCycle {
    /// keyframes the sky passes through as the day progresses
    pub frames: HashMap<String, Frame>,
    /// the length of one full cycle
    pub day_length: i32,
    /// the offset of the cycle
    pub day_offset: i32,
    /// flags for special behavior
    pub flags: i32,
    /// this environment enhancement project version
    pub env_version: i32,
    /// altitudes to track
    pub track_altitudes: Vec3,
    /// parcel ID
    pub parcel_id: i32,
    /// the ID of the region this DayCycle belongs to
    pub region_id: Uuid,
}

/// Which values this frame update
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Frame {
    /// A frame of water information
    Water(WaterFrame),
    /// A frame of sky information
    Sky(SkyFrame),
}

/// types for parsing frame types from the xml
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub enum FrameType {
    /// a water frame
    Water,
    /// a sky frame
    Sky,
}
impl FromStr for FrameType {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "water" => Ok(FrameType::Water),
            "sky" => Ok(FrameType::Sky),
            other => Err(ParseError::InvalidField(format!(
                "Unknown frame type: {}",
                other
            ))),
        }
    }
}

/// Information about a water keyframe
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WaterFrame {
    /// How blurry reflections and refractions are
    pub blur_multiplier: f64,
    /// How reflective the water is overall
    pub fresnel_offset: f64,
    /// Strength of the fresnel effect
    pub fresnel_scale: f64,
    /// How bumpy the water surface normals feel
    pub normal_scale: Vec3,
    /// Normal map texture for this water frame
    pub normal_map: Uuid,
    /// How water behaves above the surface
    pub scale_above: f64,
    /// multiplier for underwater fog intensity
    pub underwater_fog_mod: f64,
    /// color for underwter fog
    pub underwater_fog_color: Rgb<f32>,
    /// Density of underwater fog
    pub underwater_fog_density: f64,
    /// Direction of wave movement 1
    pub wave1_direction: Vec2,
    /// Direction of wave movement 2
    pub wave2_direction: Vec2,
    /// texture used for water transparency
    pub transparent_texture: Uuid,
}
impl WaterFrame {
    fn from_llsd(frame_map: &HashMap<String, LLSDValue>) -> Result<Self, ParseError> {
        Ok(Self {
            blur_multiplier: *llsd_get!(frame_map, "blur_multiplier", as_real)?,
            fresnel_offset: *llsd_get!(frame_map, "fresnel_offset", as_real)?,
            fresnel_scale: *llsd_get!(frame_map, "fresnel_scale", as_real)?,
            normal_scale: llsd_vec3!(frame_map, "normal_scale"),
            normal_map: *llsd_get!(frame_map, "normal_map", as_uuid)?,
            scale_above: *llsd_get!(frame_map, "scale_above", as_real)?,
            underwater_fog_mod: *llsd_get!(frame_map, "underwater_fog_mod", as_real)?,
            underwater_fog_color: llsd_rgb!(frame_map, "water_fog_color"),
            underwater_fog_density: *llsd_get!(frame_map, "water_fog_density", as_real)?,
            wave1_direction: llsd_vec2!(frame_map, "wave1_direction"),
            wave2_direction: llsd_vec2!(frame_map, "wave2_direction"),
            transparent_texture: *llsd_get!(frame_map, "transparent_texture", as_uuid)?,
        })
    }
}

/// Controls how light is absorbed as it travels through the atmosphere or water
/// controls the dimness and filtering of the light over distance
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AbsorptionConfig {
    /// baseline absorpion that is applied regardless of distance
    pub constant_term: f64,
    /// exponential curve of the absorption
    pub exp_scale: f64,
    /// exponential growth component for very strong depth and fog effect
    pub exp_term: f64,
    /// absorption that increases linearly over distance
    pub linear_term: f64,
    /// thickness of the absorption layer
    pub width: f64,
}
impl AbsorptionConfig {
    fn from_llsd(absorption_config_map: &HashMap<String, LLSDValue>) -> Result<Self, ParseError> {
        Ok(AbsorptionConfig {
            constant_term: *llsd_get!(absorption_config_map, "constant_term", as_real)?,
            exp_scale: *llsd_get!(absorption_config_map, "exp_scale", as_real)?,
            exp_term: *llsd_get!(absorption_config_map, "exp_term", as_real)?,
            linear_term: *llsd_get!(absorption_config_map, "linear_term", as_real)?,
            width: *llsd_get!(absorption_config_map, "width", as_real)?,
        })
    }
}

/// Legacy fog and haze model. This creates colored atmospheric haze.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LegacyHaze {
    /// Base sky lighting color. Ambient light in the atmosphere.
    pub ambient: Vec3,
    /// how much blue scattering happens in the atmosphere. More blue is clearer sky.
    pub blue_density: Vec3,
    /// Color of the sky near the horizon
    pub blue_horizon: Vec3,
    /// global fog density scaling
    pub density_multiplier: f64,
    /// how far haze extends before fading
    pub distance_multiplier: f64,
    /// how dense the haze is
    pub haze_density: f64,
    /// how dense the haze is near the horizon
    pub haze_horizon: f64,
}
impl LegacyHaze {
    fn from_llsd(legacy_haze_map: &HashMap<String, LLSDValue>) -> Result<Self, ParseError> {
        Ok(LegacyHaze {
            ambient: llsd_vec3!(legacy_haze_map, "ambient"),
            blue_density: llsd_vec3!(legacy_haze_map, "blue_density"),
            blue_horizon: llsd_vec3!(legacy_haze_map, "blue_horizon"),
            density_multiplier: *llsd_get!(legacy_haze_map, "density_multiplier", as_real)?,
            distance_multiplier: *llsd_get!(legacy_haze_map, "distance_multiplier", as_real)?,
            haze_density: *llsd_get!(legacy_haze_map, "haze_density", as_real)?,
            haze_horizon: *llsd_get!(legacy_haze_map, "haze_horizon", as_real)?,
        })
    }
}

/// configuration for mie scattering, which handles scattering light from particles
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MieConfig {
    /// direction bias of scattering
    /// things like sun glare and directionality
    pub antisotropy: f64,
    /// baseline scattering that is always applied
    pub constant_term: f64,
    /// exponential curve of scattering
    pub exp_scale: f64,
    /// exponential growth component used for very strong scattering
    pub exp_term: f64,
    /// scattering that increases linearly over distance
    pub linear_term: f64,
    /// thickness of the scattering layer
    pub width: f64,
}
impl MieConfig {
    fn from_llsd(mie_config_map: &HashMap<String, LLSDValue>) -> Result<Self, ParseError> {
        Ok(MieConfig {
            antisotropy: *llsd_get!(mie_config_map, "anisotropy", as_real)?,
            constant_term: *llsd_get!(mie_config_map, "constant_term", as_real)?,
            exp_scale: *llsd_get!(mie_config_map, "exp_scale", as_real)?,
            exp_term: *llsd_get!(mie_config_map, "exp_term", as_real)?,
            linear_term: *llsd_get!(mie_config_map, "linear_term", as_real)?,
            width: *llsd_get!(mie_config_map, "width", as_real)?,
        })
    }
}

/// Rayleigh makes blue skies, bright sunsets, and dark nights.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RayleighConfig {
    /// baseline scattering
    pub constant_term: f64,
    /// exponential curve of scattering
    pub exp_scale: f64,
    /// exponential growth component for very strong scattering
    pub exp_term: f64,
    /// directional scattering that increases linearly over distance
    pub linear_term: f64,
    /// thickness of the scattering layer
    pub width: f64,
}
impl RayleighConfig {
    fn from_llsd(rayleigh_config_map: &HashMap<String, LLSDValue>) -> Result<Self, ParseError> {
        Ok(RayleighConfig {
            constant_term: *llsd_get!(rayleigh_config_map, "constant_term", as_real)?,
            exp_scale: *llsd_get!(rayleigh_config_map, "exp_scale", as_real)?,
            exp_term: *llsd_get!(rayleigh_config_map, "exp_term", as_real)?,
            linear_term: *llsd_get!(rayleigh_config_map, "linear_term", as_real)?,
            width: *llsd_get!(rayleigh_config_map, "width", as_real)?,
        })
    }
}

/// Information about how clouds look in-world
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CloudConfig {
    /// color of the cloud
    pub color: Rgb<f32>,
    /// texture of the cloud
    pub id: Uuid,
    /// cloud thickness and noise field 1
    pub pos_density_1: Vec3,
    /// cloud thickness and noise field 2
    pub pos_density_2: Vec3,
    /// scale of the clouds
    pub scale: f64,
    /// how fast the clouds are moving
    pub scroll_rate: Vec2,
    /// cloud shadow intensity
    pub shadow: f64,
    /// randomness and noise variation on clouds
    pub variance: f64,
}

/// Information about how the moon looks in-world
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MoonConfig {
    /// brightness of the moon
    pub brightness: f64,
    /// texture of the moon
    pub id: Uuid,
    /// rotation of the moon's texture
    pub rotation: Quat,
    /// size of the moon
    pub scale: f64,
}

/// Information about how the sun looks in-world
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SunConfig {
    /// Sun angle in the sky
    pub arc_radians: f64,
    /// ID of the sun texture
    pub id: Uuid,
    /// roatation of the sun texture
    pub rotation: Quat,
    /// size of the sun
    pub scale: f64,
    /// color of the light coming from the sun
    pub sunlight_color: Rgb<f32>,
}

/// A keyframe describing information about the sky
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SkyFrame {
    /// information about the absorption of the atmosphere
    pub absorption_configs: Vec<AbsorptionConfig>,
    /// effect reference that controls light bloom
    pub bloom_id: Uuid,
    /// information about the look and behavior of clouds
    pub cloud_config: CloudConfig,
    /// vertical offset of the sky dome
    pub dome_offset: f64,
    /// radius of the sky dome
    pub dome_radius: f64,
    /// size of atmospheric moisture particles
    pub droplet_radius: f64,
    /// brightness curve
    pub gamma: f64,
    /// ambient sky light
    pub glow: Rgb<f32>,
    /// reference to sun/moon halo preset or texture
    pub halo_id: Uuid,
    /// adds icy tint to atmosphere, cold haze, and seasonal effects
    pub ice_level: f64,
    /// legacy haze info
    pub legacy_haze: LegacyHaze,
    /// maximum height of atmosphere influence
    pub max_y: f64,
    /// atmospheric humidity. Affects haze, cloud formation, and scattering strength
    pub moisture_level: f64,
    /// information about the look and behavior of the moon
    pub moon_config: MoonConfig,
    /// radius of underlying planet to handle curveature and horizon bending calculations
    pub planet_radius: f64,
    /// rainbow texture
    pub rainbow_id: Uuid,
    /// inner raidus of the bottom layer of the sky dome
    pub sky_bottom_radius: f64,
    /// outer raidus of the top layer of the sky dome
    pub sky_top_radius: f64,
    /// controls visibility of the stars at night
    pub star_brightness: f64,
    /// Information about the look and behavior of the sun
    pub sun_config: SunConfig,
    /// Mie scattering, creating dust haze and glow
    pub mie_configs: Vec<MieConfig>,
    /// Rayleigh scattering which handles air scattering
    pub rayleigh_configs: Vec<RayleighConfig>,
}
impl SkyFrame {
    fn from_llsd(frame_map: &HashMap<String, LLSDValue>) -> Result<Self, ParseError> {
        let absorption_config_array = llsd_get!(frame_map, "absorption_config", as_array)?;
        let mut absorption_configs = Vec::new();
        for config in absorption_config_array {
            let config_map = config.as_map().ok_or_else(|| {
                ParseError::InvalidField(format!("AbsorptionConfig is not a map: {:?}", config))
            })?;
            absorption_configs.push(AbsorptionConfig::from_llsd(config_map)?);
        }

        let cloud_config = CloudConfig {
            color: llsd_rgb!(frame_map, "cloud_color"),
            id: *llsd_get!(frame_map, "cloud_id", as_uuid)?,
            pos_density_1: llsd_vec3!(frame_map, "cloud_pos_density1"),
            pos_density_2: llsd_vec3!(frame_map, "cloud_pos_density2"),
            scale: *llsd_get!(frame_map, "cloud_scale", as_real)?,
            scroll_rate: llsd_vec2!(frame_map, "cloud_scroll_rate"),
            shadow: *llsd_get!(frame_map, "cloud_shadow", as_real)?,
            variance: *llsd_get!(frame_map, "cloud_variance", as_real)?,
        };

        let legacy_haze_map = llsd_get!(frame_map, "legacy_haze", as_map)?;
        let legacy_haze = LegacyHaze::from_llsd(legacy_haze_map)?;

        let moon_config = MoonConfig {
            brightness: *llsd_get!(frame_map, "moon_brightness", as_real)?,
            id: *llsd_get!(frame_map, "moon_id", as_uuid)?,
            rotation: llsd_quat!(frame_map, "moon_rotation"),
            scale: *llsd_get!(frame_map, "moon_scale", as_real)?,
        };

        let sun_config = SunConfig {
            arc_radians: *llsd_get!(frame_map, "sun_arc_radians", as_real)?,
            id: *llsd_get!(frame_map, "sun_id", as_uuid)?,
            rotation: llsd_quat!(frame_map, "sun_rotation"),
            scale: *llsd_get!(frame_map, "sun_scale", as_real)?,
            sunlight_color: llsd_rgb!(frame_map, "sunlight_color"),
        };

        let mie_config_array = llsd_get!(frame_map, "mie_config", as_array)?;
        let mut mie_configs = Vec::new();
        for config in mie_config_array {
            let config_map = config.as_map().ok_or_else(|| {
                ParseError::InvalidField(format!("MieConfig is not a map: {:?}", config))
            })?;
            mie_configs.push(MieConfig::from_llsd(config_map)?);
        }

        let rayleigh_config_array = llsd_get!(frame_map, "rayleigh_config", as_array)?;
        let mut rayleigh_configs = Vec::new();
        for config in rayleigh_config_array {
            let config_map = config.as_map().ok_or_else(|| {
                ParseError::InvalidField(format!("RayleighConfig is not a map: {:?}", config))
            })?;
            rayleigh_configs.push(RayleighConfig::from_llsd(config_map)?);
        }

        Ok(Self {
            absorption_configs,
            bloom_id: *llsd_get!(frame_map, "bloom_id", as_uuid)?,
            cloud_config,
            dome_offset: *llsd_get!(frame_map, "dome_offset", as_real)?,
            dome_radius: *llsd_get!(frame_map, "dome_radius", as_real)?,
            droplet_radius: *llsd_get!(frame_map, "droplet_radius", as_real)?,
            gamma: *llsd_get!(frame_map, "gamma", as_real)?,
            glow: llsd_rgb!(frame_map, "glow"),
            halo_id: *llsd_get!(frame_map, "halo_id", as_uuid)?,
            ice_level: *llsd_get!(frame_map, "ice_level", as_real)?,
            legacy_haze,
            max_y: *llsd_get!(frame_map, "max_y", as_real)?,
            moisture_level: *llsd_get!(frame_map, "moisture_level", as_real)?,
            moon_config,
            planet_radius: *llsd_get!(frame_map, "planet_radius", as_real)?,
            rainbow_id: *llsd_get!(frame_map, "rainbow_id", as_uuid)?,
            sky_bottom_radius: *llsd_get!(frame_map, "sky_bottom_radius", as_real)?,
            sky_top_radius: *llsd_get!(frame_map, "sky_top_radius", as_real)?,
            star_brightness: *llsd_get!(frame_map, "star_brightness", as_real)?,
            sun_config,
            mie_configs,
            rayleigh_configs,
        })
    }
}
impl DayCycle {
    /// convert environment data from bytes retreived from the endpoint
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, ParseError> {
        let data = xml::from_str(&String::from_utf8_lossy(&bytes))?;
        Self::from_llsd(data)
    }
    /// convert environment data from LLSD values
    pub fn from_llsd(data: LLSDValue) -> Result<Self, ParseError> {
        let map = data
            .as_map()
            .ok_or_else(|| ParseError::MissingField("Expected top level map".into()))?;
        let environment = llsd_get!(map, "environment", as_map)?;

        // handle day cycle map
        let day_cycle = llsd_get!(environment, "day_cycle", as_map)?;
        let frames_map = llsd_get!(day_cycle, "frames", as_map)?;
        let mut frames = HashMap::new();
        for (frame_id, frame_value) in frames_map.into_iter() {
            let frame_map = frame_value.as_map().ok_or_else(|| {
                ParseError::InvalidField(format!(
                    "Frame {} is not a map: {:?}",
                    frame_id, frame_value
                ))
            })?;
            let frame_type = FrameType::from_str(llsd_get!(frame_map, "type", as_string)?)?;
            let frame = match frame_type {
                FrameType::Water => Frame::Water(WaterFrame::from_llsd(frame_map)?),
                FrameType::Sky => Frame::Sky(SkyFrame::from_llsd(frame_map)?),
            };
            frames.insert(frame_id.clone(), frame);
        }

        Ok(DayCycle {
            day_length: *llsd_get!(environment, "day_length", as_integer)?,
            day_offset: *llsd_get!(environment, "day_offset", as_integer)?,
            flags: *llsd_get!(environment, "flags", as_integer)?,
            env_version: *llsd_get!(environment, "env_version", as_integer)?,
            track_altitudes: llsd_vec3!(environment, "track_altitudes"),
            parcel_id: *llsd_get!(environment, "parcel_id", as_integer)?,
            region_id: *llsd_get!(environment, "region_id", as_uuid)?,
            frames,
        })
    }
}

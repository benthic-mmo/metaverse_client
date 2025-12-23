use byteorder::{LittleEndian, ReadBytesExt};
use glam::{Quat, Vec3};
use rgb::Rgba;
use uuid::Uuid;

use crate::errors::ParseError;
use crate::packet::{
    header::{Header, PacketFrequency},
    packet::{Packet, PacketData},
    packet_types::PacketType,
};
use crate::udp::object::object_update::ExtraParams;
use crate::udp::object::util::ObjectFlag;
use crate::utils::material::MaterialType;
use crate::utils::object_types::ObjectType;
use crate::utils::path::Path;
use crate::utils::sound::AttachedSound;

use std::io::{Cursor, Read};

/// bitflags for compressed data flags. CompressedObjectUpdates are decoded conditionally, based on
/// the flags defined here. If these are not present, portions are not decoded.
#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CompressedFlag {
    /// Scratch pad data is included.
    ScratchPad = 0x0000_0001,
    /// Tree structure data is included.
    Tree = 0x0000_0002,
    /// Text data is present for the object.
    HasText = 0x0000_0004,
    /// Legacy particle system data is included.
    HasParticlesLegacy = 0x0000_0008,
    /// Sound data is present.
    HasSound = 0x0000_0010,
    /// The object has a parent link.
    HasParent = 0x0000_0020,
    /// Texture animation data is included.
    TextureAnimation = 0x0000_0040,
    /// Angular velocity information is present.
    HasAngularVelocity = 0x0000_0080,
    /// Name-value pairs (custom data) are included.
    HasNameValues = 0x0000_0100,
    /// Media URL is present on the object.
    MediaURL = 0x0000_0200,
    /// Particle system data is included (non-legacy format).
    HasParticles = 0x0000_0400,
}

impl CompressedFlag {
    /// convert a flag to the compressed flag enum
    pub fn from_bytes(bits: u32) -> Vec<CompressedFlag> {
        let mut flags = Vec::new();
        for &flag in [
            CompressedFlag::ScratchPad,
            CompressedFlag::Tree,
            CompressedFlag::HasText,
            CompressedFlag::HasParticlesLegacy,
            CompressedFlag::HasSound,
            CompressedFlag::HasParent,
            CompressedFlag::TextureAnimation,
            CompressedFlag::HasAngularVelocity,
            CompressedFlag::HasNameValues,
            CompressedFlag::MediaURL,
            CompressedFlag::HasParticles,
        ]
        .iter()
        {
            if bits & (flag as u32) != 0 {
                flags.push(flag);
            }
        }
        flags
    }
}
impl Packet {
    /// create a new object update compressed packet
    pub fn new_object_update_compressed(object_update_compressed: ObjectUpdateCompressed) -> Self {
        Packet {
            header: Header {
                id: 13,
                reliable: true,
                zerocoded: false,
                frequency: PacketFrequency::High,
                ..Default::default()
            },
            body: PacketType::ObjectUpdateCompressed(Box::new(object_update_compressed)),
        }
    }
}

/// ObjectUpdateCompressed fields
/// Each ObjectUpdatecompressed can contain several objects worth of data
#[derive(Debug, Clone)]
pub struct ObjectUpdateCompressed {
    /// region ID of the objects
    pub region_handle: u64,
    /// time dilation of the objects
    pub time_dilation: u16,
    /// list of objects to update
    pub object_data: Vec<ObjectDataCompressed>,
}

/// ObjectUpdateCompressed data field
#[derive(Debug, Clone)]
pub struct ObjectDataCompressed {
    /// The variable flags for decoding the packet. These flags determine which fields are present
    /// and how many bytes will ultimately be read
    pub update_flags: Vec<ObjectFlag>,
    /// full Id of the object
    pub full_id: Uuid,
    /// local ID of the object within the scene
    pub local_id: u32,
    /// type of the object
    pub pcode: ObjectType,
    /// object state
    pub state: u8,
    /// crc or pseudo crc
    pub crc: u32,
    /// material the object is made of
    pub material: MaterialType,
    /// action taken on click
    pub click_action: u8,
    /// scale of the object. For objects with parents, this will be a relative scale. For objects
    /// without parents this is a global scale.
    pub scale: Vec3,
    /// position of the object. For objects with parents, this will be a relative position. For
    /// objects without parents this is a global scale.
    pub position: Vec3,
    /// rotation of the object
    pub rotation: Quat,
    /// owner ID
    pub owner_id: Option<Uuid>,
    /// angular velocity of the object
    pub angular_velocity: Option<Vec3>,
    /// local ID of the parent within the scene
    pub parent_id: Option<u32>,
    /// Hovering text above the object
    pub text: Option<String>,
    /// text color above the object
    pub text_color: Option<Rgba<u8>>,
    /// media URL linked to the object
    pub media_url: Option<String>,
    /// legacy particle system
    pub particle_system_legacy: Option<Vec<u8>>,
    /// extra params. Contains several types of optional data, including sculpts and mesh.
    pub extra_params: Option<Vec<ExtraParams>>,
    /// sound the object emits
    pub sound: Option<AttachedSound>,
    /// name value. Used for avatar names, and storing attachment information
    pub name_values: Option<String>,
    /// path data for the object's sculpt
    pub sculpt_path: Path,
    /// texture data for the object
    pub texture_entry: Vec<u8>,
    /// texture animation data for the object
    pub texture_animation: Option<Vec<u8>>,
    /// particle system information
    pub particle_system: Vec<u8>,
}

impl PacketData for ObjectUpdateCompressed {
    fn from_bytes(bytes: &[u8]) -> Result<Self, ParseError> {
        let mut cursor = Cursor::new(bytes);
        let region_handle = cursor.read_u64::<LittleEndian>()?;
        let time_dilation = cursor.read_u16::<LittleEndian>()?;
        let object_data_length = cursor.read_u8()?;
        let mut object_data = Vec::new();
        for _ in 0..object_data_length {
            let update_flags = ObjectFlag::from_bytes(cursor.read_u32::<LittleEndian>()?);
            let _data_size = cursor.read_u16::<LittleEndian>()? as usize;

            let mut full_id_bytes = [0u8; 16];
            cursor.read_exact(&mut full_id_bytes)?;
            let full_id = Uuid::from_bytes(full_id_bytes);

            let local_id = cursor.read_u32::<LittleEndian>()?;

            let pcode = ObjectType::from_bytes(&cursor.read_u8()?);
            let state = cursor.read_u8()?;
            let crc = cursor.read_u32::<LittleEndian>()?;
            let material = MaterialType::from_bytes(&cursor.read_u8()?);
            let click_action = cursor.read_u8()?;

            let scale_x = cursor.read_f32::<LittleEndian>()?;
            let scale_y = cursor.read_f32::<LittleEndian>()?;
            let scale_z = cursor.read_f32::<LittleEndian>()?;
            let scale = Vec3 {
                x: scale_x,
                y: scale_y,
                z: scale_z,
            };
            // flip the Z and Y positions for easier rendering in other coordinate spaces
            let position_x = cursor.read_f32::<LittleEndian>()?;
            let position_y = cursor.read_f32::<LittleEndian>()?;
            let position_z = cursor.read_f32::<LittleEndian>()?;
            let position = Vec3 {
                x: position_x,
                y: position_z,
                z: position_y,
            };

            let x = cursor.read_f32::<LittleEndian>()?;
            let y = cursor.read_f32::<LittleEndian>()?;
            let z = cursor.read_f32::<LittleEndian>()?;
            let w_sq = 1.0 - x * x - y * y - z * z;
            let w = if w_sq > 0.0 { w_sq.sqrt() } else { 0.0 };
            let rotation = Quat::from_xyzw(x, y, z, w);

            let compressed_flags = CompressedFlag::from_bytes(cursor.read_u32::<LittleEndian>()?);

            let mut owner_id_bytes = [0u8; 16];
            cursor.read_exact(&mut owner_id_bytes)?;
            let owner_id = if compressed_flags.contains(&CompressedFlag::HasParticles)
                || compressed_flags.contains(&CompressedFlag::HasParticlesLegacy)
                || compressed_flags.contains(&CompressedFlag::HasSound)
            {
                Some(Uuid::from_bytes(owner_id_bytes))
            } else {
                None
            };

            let angular_velocity = if compressed_flags.contains(&CompressedFlag::HasAngularVelocity)
            {
                let angular_velocity_x = cursor.read_f32::<LittleEndian>()?;
                let angular_velocity_y = cursor.read_f32::<LittleEndian>()?;
                let angular_velocity_z = cursor.read_f32::<LittleEndian>()?;
                Some(Vec3 {
                    x: angular_velocity_x,
                    y: angular_velocity_y,
                    z: angular_velocity_z,
                })
            } else {
                None
            };

            let parent_id = if compressed_flags.contains(&CompressedFlag::HasParent) {
                Some(cursor.read_u32::<LittleEndian>()?)
            } else {
                None
            };

            let (text, text_color) = if compressed_flags.contains(&CompressedFlag::HasText) {
                let text_length = cursor.read_u8()?;
                let mut text = vec![0u8; text_length as usize];
                cursor.read_exact(&mut text)?;
                let text = String::from_utf8_lossy(&text).to_string();

                let text_color_r = cursor.read_u8()?;
                let text_color_g = cursor.read_u8()?;
                let text_color_b = cursor.read_u8()?;
                let text_color_a = cursor.read_u8()?;
                let text_color = Rgba {
                    r: text_color_r,
                    g: text_color_g,
                    b: text_color_b,
                    a: text_color_a,
                };
                (Some(text), Some(text_color))
            } else {
                (None, None)
            };

            let media_url = if compressed_flags.contains(&CompressedFlag::MediaURL) {
                let media_url_length = cursor.read_u8()?;
                let mut media_url = vec![0u8; media_url_length as usize];
                cursor.read_exact(&mut media_url)?;
                Some(String::from_utf8_lossy(&media_url).to_string())
            } else {
                None
            };

            let particle_system_legacy = if compressed_flags
                .contains(&CompressedFlag::HasParticlesLegacy)
                || compressed_flags.contains(&CompressedFlag::HasParticles)
            {
                let particle_system_block_length = cursor.read_u8()?;
                let mut particle_system_block = vec![0u8; particle_system_block_length as usize];
                cursor.read_exact(&mut particle_system_block)?;
                Some(particle_system_block)
            } else {
                None
            };

            // peek at the length of the extra params without reading the byte
            let pos = cursor.position() as usize;
            let extra_params_count = cursor.get_ref()[pos];
            let extra_params = if extra_params_count == 0 {
                None
            } else {
                let (extra_params, position) = ExtraParams::from_bytes(&cursor.get_ref()[pos..])?;
                cursor.set_position(cursor.position() + position);
                Some(extra_params)
            };

            // TODO: make the sound struct
            let sound = if compressed_flags.contains(&CompressedFlag::HasSound) {
                let mut sound_id_bytes = [0u8; 16];
                cursor.read_exact(&mut sound_id_bytes)?;
                let sound_id = Uuid::from_bytes(sound_id_bytes);
                let gain = cursor.read_f32::<LittleEndian>()?;
                let flags = cursor.read_u8()?;
                let radius = cursor.read_f32::<LittleEndian>()?;
                Some(AttachedSound {
                    owner_id: None,
                    sound_id,
                    gain,
                    flags,
                    radius,
                })
            } else {
                None
            };

            let name_values = if compressed_flags.contains(&CompressedFlag::HasNameValues) {
                let name_value_length = cursor.read_u16::<LittleEndian>()?;
                let mut name_value = vec![0u8; name_value_length as usize];
                cursor.read_exact(&mut name_value)?;
                Some(String::from_utf8_lossy(&name_value).to_string())
            } else {
                None
            };

            let mut geometry_bytes = [0u8; 23];
            cursor.read_exact(&mut geometry_bytes)?;
            let sculpt_path = Path::from_bytes(&geometry_bytes)?;

            let texture_entry_length = cursor.read_u16::<LittleEndian>()?;
            let mut texture_entry_bytes = vec![0u8; texture_entry_length as usize];
            cursor.read_exact(&mut texture_entry_bytes)?;
            let texture_entry = texture_entry_bytes;
            //let texture_entry = Texture::from_bytes(&texture_entry_bytes)?;

            let texture_animation = if compressed_flags.contains(&CompressedFlag::TextureAnimation)
            {
                let texture_anim_length = cursor.read_u8()?;
                let mut texture_anim = vec![0u8; texture_anim_length as usize];
                cursor.read_exact(&mut texture_anim)?;
                Some(texture_anim)
            } else {
                None
            };

            let particle_system_length = cursor.read_u8()?;
            let mut particle_system = vec![0u8; particle_system_length as usize];
            cursor.read_exact(&mut particle_system)?;

            // one unhandled zero at the end of the packet.
            // hopefully this isn't an issue
            cursor.read_u8()?;

            let data = ObjectDataCompressed {
                update_flags,
                full_id,
                local_id,
                pcode,
                state,
                crc,
                material,
                click_action,
                scale,
                position,
                rotation,
                owner_id,
                angular_velocity,
                parent_id,
                text,
                text_color,
                media_url,
                particle_system_legacy,
                extra_params,
                sound,
                name_values,
                sculpt_path,
                texture_entry,
                texture_animation,
                particle_system,
            };
            object_data.push(data);
        }
        Ok(ObjectUpdateCompressed {
            region_handle,
            time_dilation,
            object_data,
        })
    }
    fn to_bytes(&self) -> Vec<u8> {
        // push your data into the new vector
        Vec::new()
    }
}

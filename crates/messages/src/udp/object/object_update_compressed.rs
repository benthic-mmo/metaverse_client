use byteorder::{LittleEndian, ReadBytesExt};
use glam::{Quat, Vec3};
use rgb::Rgba;
use uuid::Uuid;

use crate::errors::ParseError;
use crate::packet::{
    header::{Header, PacketFrequency},
    packet_protocol::{Packet, PacketData},
    packet_types::PacketType,
};
use crate::parse;
use crate::udp::object::object_update::ExtraParams;
use crate::udp::object::util::{ObjectFlag, ObjectUpdateData};
use crate::utils::material::MaterialType;
use crate::utils::object_types::ObjectType;
use crate::utils::path::PrimPath;
use crate::utils::sound::AttachedSound;
use crate::utils::texture_entry::TextureEntries;

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
    /// sound the objecPrims
    pub sound: Option<AttachedSound>,
    /// name value. Used for avatar names, and storing attachment information
    pub name_values: Option<String>,
    /// path data for the object's sculpt
    pub sculpt_path: Option<PrimPath>,
    /// texture data for the object
    pub texture_entry: Option<TextureEntries>,
    /// texture animation data for the object
    pub texture_animation: Option<Vec<u8>>,
    /// particle system information
    pub particle_system: Option<Vec<u8>>,
}
impl ObjectUpdateData for ObjectDataCompressed {
    fn object_type(&self) -> ObjectType {
        self.pcode
    }
    fn full_id(&self) -> Uuid {
        self.full_id
    }
    fn parent_id(&self) -> Option<u32> {
        self.parent_id
    }
    fn local_id(&self) -> u32 {
        self.local_id
    }
    fn name_value(&self) -> &Option<String> {
        &self.name_values
    }
    fn position(&self) -> Vec3 {
        self.position
    }
    fn extra_params(&self) -> Option<&[ExtraParams]> {
        self.extra_params.as_deref()
    }
    fn rotation(&self) -> Quat {
        self.rotation
    }
    fn scale(&self) -> Vec3 {
        self.scale
    }
    fn texture(&self) -> &Option<TextureEntries> {
        &self.texture_entry
    }
    fn crc(&self) -> u32 {
        self.crc
    }
    fn sculpt_path(&self) -> Option<PrimPath> {
        self.sculpt_path.clone()
    }
}

impl PacketData for ObjectUpdateCompressed {
    fn from_bytes(bytes: &[u8]) -> Result<Self, ParseError> {
        let mut cursor = Cursor::new(bytes);

        let region_handle = parse!(cursor.read_u64::<LittleEndian>())?;

        let time_dilation = parse!(cursor.read_u16::<LittleEndian>())?;

        let object_data_length = parse!(cursor.read_u8())?;

        let mut object_data = Vec::new();

        for _ in 0..object_data_length {
            let update_flags = ObjectFlag::from_bytes(parse!(cursor.read_u32::<LittleEndian>())?);

            let data_size = parse!(cursor.read_u16::<LittleEndian>())? as usize;

            let start = cursor.position() as usize;

            let mut full_id_bytes = [0u8; 16];
            parse!(cursor.read_exact(&mut full_id_bytes))?;
            let full_id = Uuid::from_bytes(full_id_bytes);

            let local_id = parse!(cursor.read_u32::<LittleEndian>())?;

            let pcode = ObjectType::from_bytes(&parse!(cursor.read_u8())?);

            let state = parse!(cursor.read_u8())?;

            let crc = parse!(cursor.read_u32::<LittleEndian>())?;

            let material = MaterialType::from_bytes(&parse!(cursor.read_u8())?);

            let click_action = parse!(cursor.read_u8())?;

            let scale = Vec3 {
                x: parse!(cursor.read_f32::<LittleEndian>())?,
                y: parse!(cursor.read_f32::<LittleEndian>())?,
                z: parse!(cursor.read_f32::<LittleEndian>())?,
            };

            let position = Vec3 {
                x: parse!(cursor.read_f32::<LittleEndian>())?,
                y: parse!(cursor.read_f32::<LittleEndian>())?,
                z: parse!(cursor.read_f32::<LittleEndian>())?,
            };

            let x = parse!(cursor.read_f32::<LittleEndian>())?;

            let y = parse!(cursor.read_f32::<LittleEndian>())?;

            let z = parse!(cursor.read_f32::<LittleEndian>())?;

            let w_sq = 1.0 - x * x - y * y - z * z;
            let w = if w_sq > 0.0 { w_sq.sqrt() } else { 0.0 };
            let rotation = Quat::from_xyzw(x, y, z, w);

            let compressed_flags =
                CompressedFlag::from_bytes(parse!(cursor.read_u32::<LittleEndian>())?);

            let mut owner_id_bytes = [0u8; 16];
            parse!(cursor.read_exact(&mut owner_id_bytes))?;
            let owner_id = Some(Uuid::from_bytes(owner_id_bytes));

            let angular_velocity = if compressed_flags.contains(&CompressedFlag::HasAngularVelocity)
            {
                let angular_velocity = Vec3 {
                    x: parse!(cursor.read_f32::<LittleEndian>())?,
                    y: parse!(cursor.read_f32::<LittleEndian>())?,
                    z: parse!(cursor.read_f32::<LittleEndian>())?,
                };
                Some(angular_velocity)
            } else {
                None
            };

            let parent_id = if compressed_flags.contains(&CompressedFlag::HasParent) {
                let parent_id = parse!(cursor.read_u32::<LittleEndian>())?;
                Some(parent_id)
            } else {
                None
            };

            let _data = if compressed_flags.contains(&CompressedFlag::Tree) {
                let data = parse!(cursor.read_u8())?;
                Some(vec![data])
            } else if compressed_flags.contains(&CompressedFlag::ScratchPad) {
                let size = parse!(cursor.read_u32::<LittleEndian>())?;

                let mut buf = vec![0u8; size as usize];
                parse!(cursor.read_exact(&mut buf))?;

                Some(buf)
            } else {
                None
            };

            let (text, text_color) = if compressed_flags.contains(&CompressedFlag::HasText) {
                let mut text_bytes = Vec::new();

                loop {
                    let byte = parse!(cursor.read_u8())?;

                    if byte == 0 {
                        break;
                    }

                    text_bytes.push(byte);
                }

                let text = String::from_utf8_lossy(&text_bytes).to_string();

                let text_color = Rgba {
                    r: parse!(cursor.read_u8())?,
                    g: parse!(cursor.read_u8())?,
                    b: parse!(cursor.read_u8())?,
                    a: 255 - parse!(cursor.read_u8())?,
                };

                (Some(text), Some(text_color))
            } else {
                (None, None)
            };

            let media_url = if compressed_flags.contains(&CompressedFlag::MediaURL) {
                let media_url_length = parse!(cursor.read_u16::<LittleEndian>())?;

                let mut media_url = vec![0u8; media_url_length as usize];
                parse!(cursor.read_exact(&mut media_url))?;
                let media_url = String::from_utf8_lossy(&media_url).to_string();

                Some(media_url)
            } else {
                None
            };

            let particle_system_legacy =
                if compressed_flags.contains(&CompressedFlag::HasParticlesLegacy) {
                    let mut buf = [0u8; 86];
                    parse!(cursor.read_exact(&mut buf))?;
                    Some(buf.to_vec())
                } else {
                    None
                };

            let extra_params_count = parse!(cursor.read_u8())?;

            if extra_params_count != 0 {
                cursor.set_position(cursor.position() - 1);
            }

            let extra_params = if extra_params_count == 0 {
                None
            } else {
                let (extra_params, position) = parse!(ExtraParams::from_bytes(
                    &cursor.get_ref()[cursor.position() as usize..]
                ))?;

                cursor.set_position(cursor.position() + position);
                Some(extra_params)
            };

            let sound = if compressed_flags.contains(&CompressedFlag::HasSound) {
                let mut sound_id_bytes = [0u8; 16];
                parse!(cursor.read_exact(&mut sound_id_bytes))?;

                let sound_id = Uuid::from_bytes(sound_id_bytes);

                let gain = parse!(cursor.read_f32::<LittleEndian>())?;

                let flags = parse!(cursor.read_u8())?;

                let radius = parse!(cursor.read_f32::<LittleEndian>())?;

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
                let name_value_length = parse!(cursor.read_u16::<LittleEndian>())?;

                let mut name_value = vec![0u8; name_value_length as usize];
                parse!(cursor.read_exact(&mut name_value))?;

                let name_value = String::from_utf8_lossy(&name_value).to_string();

                Some(name_value)
            } else {
                None
            };

            let (sculpt_path, texture_entry, texture_animation, particle_system) = if pcode
                == ObjectType::Prim
            {
                let mut geometry_bytes = [0u8; 23];
                parse!(cursor.read_exact(&mut geometry_bytes))?;

                let sculpt_path = parse!(PrimPath::from_bytes_compressed(&geometry_bytes))?;

                let te_len = parse!(cursor.read_u16::<LittleEndian>())?;

                let mut te_bytes = vec![0u8; te_len as usize];
                parse!(cursor.read_exact(&mut te_bytes))?;

                let texture_entry = parse!(TextureEntries::from_bytes(&te_bytes))?;

                let texture_animation =
                    if compressed_flags.contains(&CompressedFlag::TextureAnimation) {
                        let len = parse!(cursor.read_u8())?;

                        let mut buf = vec![0u8; len as usize];
                        parse!(cursor.read_exact(&mut buf))?;

                        Some(buf)
                    } else {
                        None
                    };

                let particle_system = if compressed_flags.contains(&CompressedFlag::HasParticles) {
                    let sys_size = parse!(cursor.read_i32::<LittleEndian>())?;

                    let mut buf = vec![0u8; 4 + sys_size as usize];
                    buf[..4].copy_from_slice(&sys_size.to_le_bytes());

                    parse!(cursor.read_exact(&mut buf[4..]))?;

                    Some(buf)
                } else {
                    None
                };

                (
                    Some(sculpt_path),
                    Some(texture_entry),
                    texture_animation,
                    particle_system,
                )
            } else {
                (None, None, None, None)
            };

            object_data.push(ObjectDataCompressed {
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
            });

            cursor.set_position(start as u64 + data_size as u64);
        }

        Ok(ObjectUpdateCompressed {
            region_handle,
            time_dilation,
            object_data,
        })
    }

    fn to_bytes(&self) -> Vec<u8> {
        Vec::new()
    }
}

use actix::Message;
use byteorder::{LittleEndian, ReadBytesExt};
use glam::{Vec3, Vec4};
use rgb::Rgba;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    errors::ParseError,
    http::scene::SculptType,
    packet::{
        header::{Header, PacketFrequency},
        packet::{Packet, PacketData},
        packet_types::PacketType,
    },
    udp::object::util::ObjectFlag,
    utils::{material::MaterialType, object_types::ObjectType, path::Path, sound::AttachedSound},
};
use std::io::{self, Cursor, Read};

impl Packet {
    /// create a new object update packet
    pub fn new_object_update(object_update: ObjectUpdate) -> Self {
        Packet {
            header: Header {
                id: 12,
                reliable: true,
                zerocoded: true,
                frequency: PacketFrequency::High,
                ..Default::default()
            },
            body: PacketType::ObjectUpdate(Box::new(object_update)),
        }
    }
}

#[derive(Debug, Message, Clone, Serialize, Deserialize)]
#[rtype(result = "()")]
/// The object update packet. Receives object information. Is the first packet received when
/// spawning objects into the viewer.
pub struct ObjectUpdate {
    /// region handle
    pub region_handle: u64,
    /// The current lag from the server. Used by physics simulations to keep up with real time.
    pub time_dilation: f32,
    /// The region local ID of the tasks. UsedSerializeSerialize for most operations in lieu of the task's full UUID
    pub id: u32,
    /// unused except by grass. Used to determine species of grass.
    pub state: u8,
    /// Full UUID of the object
    pub full_id: Uuid,
    /// Copied directly from each message and not checked. Used for cache.
    pub crc: u32,
    /// Type of object reperesented by the task. Includes avatars, grass, trees, etc
    pub pcode: ObjectType,
    /// Type of material the object is made of.
    pub material: MaterialType,
    /// default action to take when the object is clicked on. Sit, touch, open etc.
    pub click_action: u8,
    /// The size of the object
    pub scale: Vec3,
    /// Values involving rotation, velocity and position.  
    pub motion_data: MotionData,
    /// The local ID of any object this oobject is a child of. Used for creation of object and
    /// attachments. 0 if not present.  
    pub parent_id: u32,
    /// various pieces of information about object. Stores things like empty inventory, scripted,
    /// etc
    pub update_flags: Vec<ObjectFlag>,
    /// Strores imformation about primitive geometry
    pub primitive_geometry: Path,
    /// Full property list for each object's face, including textures and colors.
    pub texture_entry: Vec<u8>,
    /// Properties to set up texture animations for each face
    pub texture_anim: Vec<u8>,
    /// Any name values specific to the object. Mostly used for avatar names.
    pub name_value: String,
    /// Generic appended data
    pub data: Vec<u8>,
    /// Text that hovers over the object
    pub text: String,
    /// Color of the text that hovers over the object
    pub text_color: Rgba<u8>,
    /// URL for the media attached to the object. Always a webpage.
    pub media_url: String,
    /// Attached particle system details
    pub particle_system_block: Vec<u8>,
    /// Data related to flexible primitives, sculpt data, or attached light data.
    pub extra_params: Option<Vec<ExtraParams>>,
    /// Sound attached to the object
    pub sound: AttachedSound,
    /// Type of joint associated with the object. Should be unused
    pub joint_type: u8,
    /// Pivot of joint associated with the object. Should be unused.
    pub joint_pivot: Vec3,
    /// Offset or axies used by certain joint types. Should be unused.
    pub joint_axis_or_anchor: Vec3,
}

impl PacketData for ObjectUpdate {
    fn from_bytes(bytes: &[u8]) -> Result<Self, ParseError> {
        let mut cursor = Cursor::new(bytes);

        // read the regionhandle as two u32s instead of one u64
        let region_handle = cursor.read_u64::<LittleEndian>()?;
        let time_dilation = cursor.read_u16::<LittleEndian>()? as f32 / 65535.0;

        // unsure what this is, but this sets the correct packet alignment
        let _offset = cursor.read_u8()?;
        let id = cursor.read_u32::<LittleEndian>()?;
        let state = cursor.read_u8()?;

        let mut full_id_bytes = [0u8; 16];
        cursor.read_exact(&mut full_id_bytes)?;
        let full_id = Uuid::from_bytes(full_id_bytes);

        let crc = cursor.read_u32::<LittleEndian>()?;

        let pcode = ObjectType::from_bytes(&cursor.read_u8()?);
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

        // for Patch objects, this is always 60.
        let motion_data_length = cursor.read_u8()?;
        let mut motion_data = vec![0u8; motion_data_length as usize];
        cursor.read_exact(&mut motion_data)?;
        let motion_data = MotionData::from_bytes(&motion_data)?;

        let parent_id = cursor.read_u32::<LittleEndian>()?;
        let update_flags = ObjectFlag::from_bytes(cursor.read_u32::<LittleEndian>()?);

        // this section is always 23 bytes
        let mut geometry_bytes = [0u8; 23];
        cursor.read_exact(&mut geometry_bytes)?;
        let primitive_geometry = Path::from_bytes(&geometry_bytes)?;

        let texture_entry_length = cursor.read_u16::<LittleEndian>()?;
        let mut texture_entry_bytes = vec![0u8; texture_entry_length as usize];
        cursor.read_exact(&mut texture_entry_bytes)?;
        let texture_entry = texture_entry_bytes;
        //let texture_entry = Texture::from_bytes(&texture_entry_bytes)?;

        let texture_anim_length = cursor.read_u8()?;
        let mut texture_anim = vec![0u8; texture_anim_length as usize];
        cursor.read_exact(&mut texture_anim)?;

        let name_value_length = cursor.read_u16::<LittleEndian>()?;
        let mut name_value = vec![0u8; name_value_length as usize];
        cursor.read_exact(&mut name_value)?;
        let name_value = String::from_utf8_lossy(&name_value).to_string();

        let data_length = cursor.read_u16::<LittleEndian>()?;
        let mut data = vec![0u8; data_length as usize];
        cursor.read_exact(&mut data)?;

        let text_length = cursor.read_u16::<LittleEndian>()?;
        // only read the text color if there is text
        let (text, text_color) = if text_length != 0 {
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
            (text, text_color)
        } else {
            // the protocol pads this to 5 bytes if there is no data
            cursor.read_exact(&mut [0u8; 3])?;
            ("".to_string(), Rgba::new(0, 0, 0, 0))
        };

        let media_url_length = cursor.read_u8()?;
        let mut media_url = vec![0u8; media_url_length as usize];
        cursor.read_exact(&mut media_url)?;
        let media_url = String::from_utf8_lossy(&media_url).to_string();

        let particle_system_block_length = cursor.read_u8()?;
        let mut particle_system_block = vec![0u8; particle_system_block_length as usize];
        cursor.read_exact(&mut particle_system_block)?;

        let extra_params_length = cursor.read_u8()?;
        let extra_params = if extra_params_length > 0 {
            let mut extra_params_bytes = vec![0u8; extra_params_length as usize];
            cursor.read_exact(&mut extra_params_bytes)?;
            Some(ExtraParams::from_bytes(&extra_params_bytes)?)
        } else {
            None
        };
        let mut sound_bytes = [0u8; 41];
        cursor.read_exact(&mut sound_bytes)?;
        let sound = AttachedSound::from_bytes(&sound_bytes)?;

        let joint_type = cursor.read_u8()?;
        let joint_pivot_x = cursor.read_f32::<LittleEndian>()?;
        let joint_pivot_y = cursor.read_f32::<LittleEndian>()?;
        let joint_pivot_z = cursor.read_f32::<LittleEndian>()?;
        let joint_pivot = Vec3 {
            x: joint_pivot_x,
            y: joint_pivot_y,
            z: joint_pivot_z,
        };

        let joint_axis_or_anchor_x = cursor.read_f32::<LittleEndian>()?;
        let joint_axis_or_anchor_y = cursor.read_f32::<LittleEndian>()?;
        let joint_axis_or_anchor_z = cursor.read_f32::<LittleEndian>()?;
        let joint_axis_or_anchor = Vec3 {
            x: joint_axis_or_anchor_x,
            y: joint_axis_or_anchor_y,
            z: joint_axis_or_anchor_z,
        };

        let update = ObjectUpdate {
            region_handle,
            time_dilation,

            id,
            state,
            full_id,
            crc,
            pcode,
            click_action,
            scale,
            material,
            motion_data,
            parent_id,
            update_flags,
            primitive_geometry,
            texture_entry,
            texture_anim,
            name_value,
            data,
            text,
            text_color,
            media_url,
            particle_system_block,
            extra_params,
            sound,
            joint_type,
            joint_pivot,
            joint_axis_or_anchor,
        };
        Ok(update)
    }

    fn to_bytes(&self) -> Vec<u8> {
        Vec::new()
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
/// Stores ObjectUpdate update fields
/// This contains information about the position, velocity, acceleration and etc of the object.
/// Stores all values as f32s, despite them coming in as variable length values.
pub struct MotionData {
    /// The collision plane for setting the user's foot angle
    pub foot_collision_plane: Option<Vec4>,
    /// The location of the object in the world  
    pub position: Vec3,
    /// The speed at which the object is moving
    pub velocity: Vec3,
    /// How fast the object is accelerating
    pub acceleration: Vec3,
    /// The roatation of the object
    pub rotation: Vec3,
    /// The angular velocity of the object
    pub angular_velocity: Vec3,
}
impl MotionData {
    /// Matches the length of the data to the correct parsing function
    pub fn from_bytes(bytes: &[u8]) -> io::Result<Self> {
        match bytes.len() {
            76 => Ok(Self::from_bytes_foot_collision_high(bytes)?),
            60 => Ok(Self::from_bytes_high(bytes)?),
            48 => Ok(Self::from_bytes_foot_collision_medium(bytes)?),
            32 => Ok(Self::from_bytes_medium(bytes)?),
            16 => Ok(Self::from_bytes_low(bytes)?),
            _ => Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Incorrect MotionData size",
            )),
        }
    }
    fn from_bytes_foot_collision_high(bytes: &[u8]) -> io::Result<Self> {
        let mut cursor = Cursor::new(bytes);
        let collision_plane = Vec4::new(
            cursor.read_f32::<LittleEndian>()?,
            cursor.read_f32::<LittleEndian>()?,
            cursor.read_f32::<LittleEndian>()?,
            cursor.read_f32::<LittleEndian>()?,
        );

        let mut update_bytes = [0u8; 60];
        cursor.read_exact(&mut update_bytes)?;
        let mut update = Self::from_bytes_high(&update_bytes)?;
        update.foot_collision_plane = Some(collision_plane);
        Ok(update)
    }

    fn from_bytes_foot_collision_medium(bytes: &[u8]) -> io::Result<Self> {
        let mut cursor = Cursor::new(bytes);
        let collision_plane = Vec4::new(
            cursor.read_f32::<LittleEndian>()?,
            cursor.read_f32::<LittleEndian>()?,
            cursor.read_f32::<LittleEndian>()?,
            cursor.read_f32::<LittleEndian>()?,
        );

        let mut update_bytes = [0u8; 32];
        cursor.read_exact(&mut update_bytes)?;
        let mut update = Self::from_bytes_medium(&update_bytes)?;
        update.foot_collision_plane = Some(collision_plane);
        Ok(update)
    }

    fn from_bytes_high(bytes: &[u8]) -> io::Result<Self> {
        let mut cursor = Cursor::new(bytes);
        let x = cursor.read_f32::<LittleEndian>()?;
        let y = cursor.read_f32::<LittleEndian>()?;
        let z = cursor.read_f32::<LittleEndian>()?;

        // this allows objects to render in the right place in other coordinate systems.
        // I know this looks wrong, but it's right.
        let position = Vec3::new(x, z, y);

        let velocity = Vec3::new(
            cursor.read_f32::<LittleEndian>()?,
            cursor.read_f32::<LittleEndian>()?,
            cursor.read_f32::<LittleEndian>()?,
        );

        let acceleration = Vec3::new(
            cursor.read_f32::<LittleEndian>()?,
            cursor.read_f32::<LittleEndian>()?,
            cursor.read_f32::<LittleEndian>()?,
        );

        let rotation = Vec3::new(
            cursor.read_f32::<LittleEndian>()?,
            cursor.read_f32::<LittleEndian>()?,
            cursor.read_f32::<LittleEndian>()?,
        );

        let angular_velocity = Vec3::new(
            cursor.read_f32::<LittleEndian>()?,
            cursor.read_f32::<LittleEndian>()?,
            cursor.read_f32::<LittleEndian>()?,
        );

        Ok(Self {
            foot_collision_plane: None,
            position,
            velocity,
            acceleration,
            rotation,
            angular_velocity,
        })
    }
    fn from_bytes_medium(bytes: &[u8]) -> io::Result<Self> {
        let mut cursor = Cursor::new(bytes);
        let position = Vec3::new(
            cursor.read_u16::<LittleEndian>()? as f32,
            cursor.read_u16::<LittleEndian>()? as f32,
            cursor.read_u16::<LittleEndian>()? as f32,
        );

        let velocity = Vec3::new(
            cursor.read_u16::<LittleEndian>()? as f32,
            cursor.read_u16::<LittleEndian>()? as f32,
            cursor.read_u16::<LittleEndian>()? as f32,
        );

        let acceleration = Vec3::new(
            cursor.read_u16::<LittleEndian>()? as f32,
            cursor.read_u16::<LittleEndian>()? as f32,
            cursor.read_u16::<LittleEndian>()? as f32,
        );

        let rotation = Vec3::new(
            cursor.read_u16::<LittleEndian>()? as f32,
            cursor.read_u16::<LittleEndian>()? as f32,
            cursor.read_u16::<LittleEndian>()? as f32,
        );

        let angular_velocity = Vec3::new(
            cursor.read_u16::<LittleEndian>()? as f32,
            cursor.read_u16::<LittleEndian>()? as f32,
            cursor.read_u16::<LittleEndian>()? as f32,
        );
        Ok(Self {
            foot_collision_plane: None,
            position,
            velocity,
            acceleration,
            rotation,
            angular_velocity,
        })
    }
    fn from_bytes_low(bytes: &[u8]) -> io::Result<Self> {
        let mut cursor = Cursor::new(bytes);
        let position = Vec3::new(
            cursor.read_u8()? as f32,
            cursor.read_u8()? as f32,
            cursor.read_u8()? as f32,
        );

        let velocity = Vec3::new(
            cursor.read_u8()? as f32,
            cursor.read_u8()? as f32,
            cursor.read_u8()? as f32,
        );

        let acceleration = Vec3::new(
            cursor.read_u8()? as f32,
            cursor.read_u8()? as f32,
            cursor.read_u8()? as f32,
        );

        let rotation = Vec3::new(
            cursor.read_u8()? as f32,
            cursor.read_u8()? as f32,
            cursor.read_u8()? as f32,
        );

        let angular_velocity = Vec3::new(
            cursor.read_u8()? as f32,
            cursor.read_u8()? as f32,
            cursor.read_u8()? as f32,
        );
        Ok(Self {
            foot_collision_plane: None,
            position,
            velocity,
            acceleration,
            rotation,
            angular_velocity,
        })
    }
}
#[derive(Debug)]
pub enum ParamTypeTag {
    Flexi,
    Light,
    Sculpt,
    Projection,
    MeshFlags,
    Materials,
    ReflectionProbe,
    Unknown,
}

impl ParamTypeTag {
    fn from_bytes(byte: &u8) -> Self {
        match byte {
            16 => ParamTypeTag::Flexi,
            32 => ParamTypeTag::Light,
            48 => ParamTypeTag::Sculpt,
            64 => ParamTypeTag::Projection,
            112 => ParamTypeTag::MeshFlags,
            128 => ParamTypeTag::Materials,
            144 => ParamTypeTag::ReflectionProbe,
            _ => ParamTypeTag::Unknown,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExtraParams {
    Flexi(FlexiData),
    Light(LightData),
    Sculpt(SculptData),
    Projection(ProjectionData),
    MeshFlags(MeshFlagsData),
    Materials(MaterialsData),
    ReflectionProbe(ReflectionProbeData),
    Unknown(UnknownData),
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SculptData {
    pub texture_id: Uuid,
    pub sculpt_type: SculptType,
}
impl SculptData {
    pub fn from_bytes(bytes: &[u8]) -> io::Result<Self> {
        let mut cursor = Cursor::new(bytes);
        let mut texture_id_bytes = [0u8; 16];
        cursor.read_exact(&mut texture_id_bytes)?;
        let texture_id = Uuid::from_bytes(texture_id_bytes);

        let sculpt_type = SculptType::from_bytes(&cursor.read_u8()?);
        Ok(SculptData {
            texture_id,
            sculpt_type,
        })
    }
}
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
///TODO: UNIMPLEMENTED
pub struct FlexiData {
    ///TODO: UNIMPLEMENTED
    pub bytes: Vec<u8>,
}
impl FlexiData {
    ///TODO: UNIMPLEMENTED
    pub fn from_bytes(bytes: &[u8]) -> io::Result<Self> {
        Ok(FlexiData {
            bytes: bytes.to_vec(),
        })
    }
}
#[derive(Debug, Clone, Default, Serialize, Deserialize)]

///TODO: UNIMPLEMENTED
pub struct LightData {
    ///TODO: UNIMPLEMENTED
    pub bytes: Vec<u8>,
}
impl LightData {
    ///TODO: UNIMPLEMENTED
    pub fn from_bytes(bytes: &[u8]) -> io::Result<Self> {
        Ok(LightData {
            bytes: bytes.to_vec(),
        })
    }
}
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
///TODO: UNIMPLEMENTED
pub struct ProjectionData {
    ///TODO: UNIMPLEMENTED
    pub bytes: Vec<u8>,
}
impl ProjectionData {
    ///TODO: UNIMPLEMENTED
    pub fn from_bytes(bytes: &[u8]) -> io::Result<Self> {
        Ok(ProjectionData {
            bytes: bytes.to_vec(),
        })
    }
}
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
///TODO: UNIMPLEMENTED
pub struct MeshFlagsData {
    ///TODO: UNIMPLEMENTED
    pub bytes: Vec<u8>,
}
impl MeshFlagsData {
    ///TODO: UNIMPLEMENTED
    pub fn from_bytes(bytes: &[u8]) -> io::Result<Self> {
        Ok(MeshFlagsData {
            bytes: bytes.to_vec(),
        })
    }
}
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
///TODO: UNIMPLEMENTED
pub struct MaterialsData {
    ///TODO: UNIMPLEMENTED
    pub bytes: Vec<u8>,
}
impl MaterialsData {
    ///TODO: UNIMPLEMENTED
    pub fn from_bytes(bytes: &[u8]) -> io::Result<Self> {
        Ok(MaterialsData {
            bytes: bytes.to_vec(),
        })
    }
}
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
///TODO: UNIMPLEMENTED
pub struct ReflectionProbeData {
    ///TODO: UNIMPLEMENTED
    pub bytes: Vec<u8>,
}
impl ReflectionProbeData {
    ///TODO: UNIMPLEMENTED
    pub fn from_bytes(bytes: &[u8]) -> io::Result<Self> {
        Ok(ReflectionProbeData {
            bytes: bytes.to_vec(),
        })
    }
}
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct UnknownData {
    pub bytes: Vec<u8>,
}
impl UnknownData {
    pub fn from_bytes(bytes: &[u8]) -> io::Result<Self> {
        Ok(UnknownData {
            bytes: bytes.to_vec(),
        })
    }
}
impl Default for ExtraParams {
    fn default() -> Self {
        ExtraParams::Unknown(UnknownData { bytes: Vec::new() })
    }
}

impl ExtraParams {
    pub fn from_bytes(bytes: &[u8]) -> io::Result<Vec<Self>> {
        let mut cursor = Cursor::new(bytes);
        let extra_params_count = match cursor.read_u8() {
            Ok(v) => v,
            Err(_) => return Ok(vec![ExtraParams::default()]),
        };
        let mut extra_params = Vec::new();
        for _ in 0..extra_params_count {
            let param_type_tag = ParamTypeTag::from_bytes(&cursor.read_u8()?);

            // padding byte
            cursor.read_u8()?;

            let param_length = cursor.read_u8()?;

            // three padding bytes
            cursor.read_u8()?;
            cursor.read_u8()?;
            cursor.read_u8()?;
            let mut param_data = vec![0u8; param_length as usize];
            cursor.read_exact(&mut param_data)?;

            let param = match param_type_tag {
                ParamTypeTag::Flexi => ExtraParams::Flexi(FlexiData::from_bytes(&param_data)?),
                ParamTypeTag::Light => ExtraParams::Light(LightData::from_bytes(&param_data)?),
                ParamTypeTag::Sculpt => ExtraParams::Sculpt(SculptData::from_bytes(&param_data)?),
                ParamTypeTag::Projection => {
                    ExtraParams::Projection(ProjectionData::from_bytes(&param_data)?)
                }
                ParamTypeTag::MeshFlags => {
                    ExtraParams::MeshFlags(MeshFlagsData::from_bytes(&param_data)?)
                }
                ParamTypeTag::Materials => {
                    ExtraParams::Materials(MaterialsData::from_bytes(&param_data)?)
                }
                ParamTypeTag::ReflectionProbe => {
                    ExtraParams::ReflectionProbe(ReflectionProbeData::from_bytes(&param_data)?)
                }
                ParamTypeTag::Unknown => {
                    ExtraParams::Unknown(UnknownData::from_bytes(&param_data)?)
                }
            };
            extra_params.push(param);
        }
        Ok(extra_params)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// the access levels found in AttachItem data
pub enum Access {
    /// user is unable to modify
    ReadOnly,
    /// user is able to write but not read the contents
    WriteOnly,
    /// user is able to both modify and read
    ReadWrite,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// the scope of the AttachItem data
pub enum Scope {
    /// object is shared globally, and not owned by a user
    Global,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// Objects that are attached to other objects contain AttachItem data in the ObjectUpdate name.
pub struct AttachItem {
    /// the ID of the object that can be used to look up this object in the user's inventory
    pub id: Uuid,
    /// the user's access level of the object
    pub access: Access,
    /// the object's scope
    pub scope: Scope,
}

impl AttachItem {
    /// Primitive objects rezzed from inventory have their metadata stored in a reference to the
    /// object in the user or the global inventory. This is stored in a string that needs to be
    /// parsed.
    pub fn parse_attach_item(data: String) -> Result<Self, ParseError> {
        let parts: Vec<&str> = data.split_whitespace().collect();
        if parts.len() != 5 {
            return Err(ParseError::Message(format!(
                "AttachItem has incorrect length: {:?}",
                data,
            )));
        }
        let id_str = parts[4].trim_end_matches('\0');
        let id = Uuid::parse_str(id_str)?;
        let access = match parts[2] {
            "RW" => Access::ReadWrite,
            "R" => Access::ReadOnly,
            "W" => Access::WriteOnly,
            _ => {
                return Err(ParseError::Message(format!(
                    "AttachItem has incorrect access value: {:?}, {:?}",
                    parts[3], data
                )))
            }
        };

        let scope = match parts[3] {
            "SV" => Scope::Global,
            _ => {
                return Err(ParseError::Message(format!(
                    "AttachItem has incorrect scope value: {:?}, {:?}",
                    parts[2], data
                )))
            }
        };
        Ok(AttachItem { id, scope, access })
    }
}

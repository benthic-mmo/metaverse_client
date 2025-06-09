use actix::Message;
use byteorder::{BigEndian, LittleEndian, ReadBytesExt};
use glam::{Vec3, Vec4};
use rgb::Rgba;
use uuid::Uuid;

use crate::{
    packet::{
        header::{Header, PacketFrequency},
        packet::{Packet, PacketData},
        packet_types::PacketType,
    },
    utils::object_types::ObjectType,
};
use std::io::{self, Cursor, Read};

impl Packet {
    /// create a new object update packet
    pub fn new_object_update(object_update: ObjectUpdate) -> Self {
        Packet {
            header: Header {
                id: 12,
                reliable: true,
                resent: false,
                zerocoded: true,
                appended_acks: false,
                sequence_number: 0,
                frequency: PacketFrequency::High,
                ack_list: None,
                size: None,
            },
            body: PacketType::ObjectUpdate(Box::new(object_update)),
        }
    }
}

#[derive(Debug, Clone, Default)]
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

#[derive(Debug, Message, Clone, Default)]
#[rtype(result = "()")]
/// The object update packet. Receives object information. Is the first packet received when
/// spawning objects into the viewer.
pub struct ObjectUpdate {
    /// x Location of the object in the grid region
    pub region_x: u32,
    /// Y location of the object in the grid region
    pub region_y: u32,
    /// The current lag from the server. Used by physics simulations to keep up with real time.
    pub time_dilation: f32,
    /// The region local ID of the tasks. Used for most operations in lieu of the task's full UUID
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
    pub update_flags: u32,
    /// Strores imformation about primitive geometry
    pub primitive_geometry: PrimitiveGeometry,
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
    pub extra_params: Vec<u8>,

    /// Sound attached to the object
    pub sound: Sound,

    /// Type of joint associated with the object. Should be unused
    pub joint_type: u8,
    /// Pivot of joint associated with the object. Should be unused.
    pub joint_pivot: Vec3,
    /// Offset or axies used by certain joint types. Should be unused.
    pub joint_axis_or_anchor: Vec3,
}

impl PacketData for ObjectUpdate {
    fn from_bytes(bytes: &[u8]) -> io::Result<Self> {
        let mut cursor = Cursor::new(bytes);

        // read the regionhandle as two u32s instead of one u64
        let region_x = cursor.read_u32::<LittleEndian>()?;
        let region_y = cursor.read_u32::<LittleEndian>()?;
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

        let motion_data_length = cursor.read_u8()?;
        let mut motion_data = vec![0u8; motion_data_length as usize];
        cursor.read_exact(&mut motion_data)?;
        let motion_data = MotionData::from_bytes(&motion_data)?;

        let parent_id = cursor.read_u32::<LittleEndian>()?;
        let update_flags = cursor.read_u32::<LittleEndian>()?;

        let mut geometry_bytes = [0u8; 23];
        cursor.read_exact(&mut geometry_bytes)?;
        let primitive_geometry = PrimitiveGeometry::from_bytes(&geometry_bytes)?;
        let texture_entry_length = cursor.read_u8()?;

        let mut texture_entry = vec![0u8; texture_entry_length as usize];
        cursor.read_exact(&mut texture_entry)?;

        let texture_anim_length = cursor.read_u8()?;
        let mut texture_anim = vec![0u8; texture_anim_length as usize];
        cursor.read_exact(&mut texture_anim)?;

        let name_value_length = cursor.read_u16::<BigEndian>()?;
        let mut name_value = vec![0u8; name_value_length as usize];
        cursor.read_exact(&mut name_value)?;
        let name_value = String::from_utf8_lossy(&name_value).to_string();

        let data_length = cursor.read_u16::<BigEndian>()?;
        let mut data = vec![0u8; data_length as usize];
        cursor.read_exact(&mut data)?;

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
        let media_url_length = cursor.read_u8()?;
        let mut media_url = vec![0u8; media_url_length as usize];
        cursor.read_exact(&mut media_url)?;
        let media_url = String::from_utf8_lossy(&media_url).to_string();

        let particle_system_block_length = cursor.read_u8()?;
        let mut particle_system_block = vec![0u8; particle_system_block_length as usize];
        cursor.read_exact(&mut particle_system_block)?;

        let extra_params_length = cursor.read_u8()?;
        let mut extra_params = vec![0u8; extra_params_length as usize];
        cursor.read_exact(&mut extra_params)?;

        let mut sound_bytes = [0u8; 41];
        cursor.read_exact(&mut sound_bytes)?;
        let sound = Sound::from_bytes(&sound_bytes)?;

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
            region_x,
            region_y,
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

#[derive(Debug, Clone, Default)]
/// Handles sounds attached to the object
pub struct Sound {
    /// Asset UUID of any attached looped sounds
    pub sound_id: Uuid,
    /// UUID of the owner of the object. Null if there is no looped sound or particle system
    /// attached to the object.
    pub owner_id: Uuid,
    /// Gain of the attached sound
    pub gain: f32,
    /// Stores flags related to attached sounds
    pub flags: u8,
    /// Radius from the center of the object that the sound should be audible from
    pub radius: f32,
}
impl Sound {
    /// Convert bytes into a Sound object
    pub fn from_bytes(bytes: &[u8]) -> std::io::Result<Self> {
        let mut cursor = Cursor::new(bytes);
        let mut sound_id_bytes = [0u8; 16];
        cursor.read_exact(&mut sound_id_bytes)?;

        let mut owner_id_bytes = [0u8; 16];
        cursor.read_exact(&mut owner_id_bytes)?;

        Ok(Self {
            sound_id: Uuid::from_bytes(sound_id_bytes),
            owner_id: Uuid::from_bytes(owner_id_bytes),
            gain: cursor.read_f32::<LittleEndian>()?,
            flags: cursor.read_u8()?,
            radius: cursor.read_f32::<LittleEndian>()?,
        })
    }
}

#[derive(Debug, Clone, Default)]
/// This contains primitive geometry information. This contains information about how a basic shape
/// can be stretched, tapered, twisted, sheared and deformed.
pub struct PrimitiveGeometry {
    /// This determines the type of path the shape follows.
    /// if it is a straight line, a circle, or etc
    /// 0x00 is Linear,
    /// 0x10 is Circular
    /// 0x20 is a flexible path
    pub path_curve: u8,
    /// The start point of the path. Controls hwo much of the extrustion is used and cuts off parts
    /// of the shape along the path.
    pub path_begin: u16,
    /// The end point of the path.
    pub path_end: u16,
    /// x Scaling at the end of the extrusion. 128 means no scaling.
    pub path_scale_x: u8,
    /// y Scaling at the end of the extrusion. 128 means no scaling.
    pub path_scale_y: u8,
    /// x axis shear. 128 is no shear.  
    pub path_shear_x: u8,
    /// y axis shear. 128 is no shear.
    pub path_shear_y: u8,
    /// twist applied at the end of the path. 128 is no twist.
    pub path_twist_end: i8,
    /// twist applied at the beginning of the path. 128 is no twist.
    pub path_twist_begin: i8,
    /// how much the shape's path moves away from the axis. 128 is no offset.
    pub path_radius_offset: i8,
    /// Tapers the shape from thick to thin on y axis. 128 is no taper.
    pub path_taper_y: i8,
    /// tapers the shape from thick to thin on x axis. 128 is no taper.
    pub path_taper_x: i8,
    /// number of times the shape revolves around its path. 128 is one revolution.
    pub path_revolutions: u8,
    /// skews the shape along its path. 128 is no skew.
    pub path_skew: i8,
    /// What the shape looks like from profile
    /// 0x00 is a circle
    /// 0x01 is a square
    /// 0x02 is a triangle
    pub profile_curve: u8,
    /// The start point of the profile. Controls how much extrusion is used and cuts off parts of
    /// the shape horizontally along the profile.
    pub profile_begin: u16,
    /// the end point of the profile.
    pub profile_end: u16,
    /// Makes a hollow in the shape. EG, a hollow cylinder becomes a tube.
    pub profile_hollow: f32,
    /// Optional object update info populated from the mesh.
    /// The shape of the hollow made into the object
    pub hollow_shape: Option<String>,
    /// Optional object update info populated from the mesh.
    /// The shape of the profile of the object.
    pub profile_shape: Option<String>,
}

impl PrimitiveGeometry {
    /// converts bytes into a PrimitiveGeometry object
    pub fn from_bytes(bytes: &[u8]) -> std::io::Result<Self> {
        let mut cursor = Cursor::new(bytes);
        Ok(Self {
            path_curve: cursor.read_u8()?,
            profile_curve: cursor.read_u8()?,
            path_begin: cursor.read_u16::<LittleEndian>()?,
            path_end: cursor.read_u16::<LittleEndian>()?,
            path_scale_x: cursor.read_u8()?,
            path_scale_y: cursor.read_u8()?,
            path_shear_x: cursor.read_u8()?,
            path_shear_y: cursor.read_u8()?,
            path_twist_end: cursor.read_i8()?,
            path_twist_begin: cursor.read_i8()?,
            path_radius_offset: cursor.read_i8()?,
            path_taper_x: cursor.read_i8()?,
            path_taper_y: cursor.read_i8()?,
            path_revolutions: cursor.read_u8()?,
            path_skew: cursor.read_i8()?,
            profile_begin: cursor.read_u16::<LittleEndian>()?,
            profile_end: cursor.read_u16::<LittleEndian>()?,
            profile_hollow: cursor.read_u16::<LittleEndian>()? as f32 / 500.0,
            hollow_shape: None,
            profile_shape: None,
        })
    }
}

#[derive(Debug, Clone, Default)]
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
                "Unknown ObjectData size",
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
        let position = Vec3::new(
            cursor.read_f32::<LittleEndian>()?,
            cursor.read_f32::<LittleEndian>()?,
            cursor.read_f32::<LittleEndian>()?,
        );

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

        let angular_velocity = Vec3::new(0.0, 0.0, 0.0);
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

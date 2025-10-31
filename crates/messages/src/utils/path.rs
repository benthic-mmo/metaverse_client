use std::io::Cursor;

use byteorder::{LittleEndian, ReadBytesExt};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
/// This contains path information. This contains information about how a basic shape
/// can be stretched, tapered, twisted, sheared and deformed.
pub struct Path {
    /// This determines the type of path the shape follows.
    /// if it is a straight line, a circle, or etc
    /// 0x00 is Linear,
    /// 0x10 is Circular
    /// 0x20 is a flexible path
    pub curve: u8,
    /// The start point of the path. Controls hwo much of the extrustion is used and cuts off parts
    /// of the shape along the path.
    pub begin: u16,
    /// The end point of the path.
    pub end: u16,
    /// x Scaling at the end of the extrusion. 128 means no scaling.
    pub scale_x: u8,
    /// y Scaling at the end of the extrusion. 128 means no scaling.
    pub scale_y: u8,
    /// x axis shear. 128 is no shear.  
    pub shear_x: u8,
    /// y axis shear. 128 is no shear.
    pub shear_y: u8,
    /// twist applied at the end of the path. 128 is no twist.
    pub twist_end: i8,
    /// twist applied at the beginning of the path. 128 is no twist.
    pub twist_begin: i8,
    /// how much the shape's path moves away from the axis. 128 is no offset.
    pub radius_offset: i8,
    /// Tapers the shape from thick to thin on y axis. 128 is no taper.
    pub taper_y: i8,
    /// tapers the shape from thick to thin on x axis. 128 is no taper.
    pub taper_x: i8,
    /// number of times the shape revolves around its path. 128 is one revolution.
    pub revolutions: u8,
    /// skews the shape along its path. 128 is no skew.
    pub skew: i8,
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

impl Path {
    /// converts bytes into a PrimitiveGeometry object
    pub fn from_bytes(bytes: &[u8]) -> std::io::Result<Self> {
        let mut cursor = Cursor::new(bytes);
        Ok(Self {
            curve: cursor.read_u8()?,
            profile_curve: cursor.read_u8()?,
            begin: cursor.read_u16::<LittleEndian>()?,
            end: cursor.read_u16::<LittleEndian>()?,
            scale_x: cursor.read_u8()?,
            scale_y: cursor.read_u8()?,
            shear_x: cursor.read_u8()?,
            shear_y: cursor.read_u8()?,
            twist_end: cursor.read_i8()?,
            twist_begin: cursor.read_i8()?,
            radius_offset: cursor.read_i8()?,
            taper_x: cursor.read_i8()?,
            taper_y: cursor.read_i8()?,
            revolutions: cursor.read_u8()?,
            skew: cursor.read_i8()?,
            profile_begin: cursor.read_u16::<LittleEndian>()?,
            profile_end: cursor.read_u16::<LittleEndian>()?,
            profile_hollow: cursor.read_u16::<LittleEndian>()? as f32 / 500.0,
            hollow_shape: None,
            profile_shape: None,
        })
    }
}

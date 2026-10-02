use crate::errors::ParseError;
use crate::packet::{
    header::{Header, PacketFrequency},
    packet_protocol::{Packet, PacketData},
    packet_types::PacketType,
};
use crate::parse;
use byteorder::{LittleEndian, ReadBytesExt};
use glam::{Quat, Vec3, Vec4};
use std::io::{Cursor, Read};

impl Packet {
    /// create a new improved terse object update packet
    pub fn new_improved_terse_object_update(
        improved_terse_object_update: ImprovedTerseObjectUpdate,
    ) -> Self {
        Packet {
            header: Header {
                id: 15,
                reliable: true,
                zerocoded: false,
                frequency: PacketFrequency::High,
                ..Default::default()
            },
            body: PacketType::ImprovedTerseObjectUpdate(Box::new(improved_terse_object_update)),
        }
    }
}

/// ImprovedTerseObjectUpdate packets are sent when objects are moved around the scen. They only
/// contain the object ID and movement data.
#[derive(Debug, Clone, Default)]
pub struct ImprovedTerseObjectUpdate {
    /// region handle for which region the object is in
    pub region_handle: u64,
    /// time dilation for the region
    pub time_dilation: u16,
    /// list of objects to update locations of
    pub objects: Vec<TerseObjectData>,
}

/// Data for object movement
#[derive(Debug, Clone)]
pub struct TerseObjectData {
    /// object's scene-local ID
    pub local_id: u32,
    /// object's current state
    pub state: u8,
    /// is the object an avatar?
    pub avatar: bool,
    /// collision plane information
    pub collision_plane: Option<Vec4>,
    /// position information
    pub position: Vec3,
    /// velocity information
    pub velocity: Vec3,
    /// acceleration information
    pub acceleration: Vec3,
    /// rotation information
    pub rotation: Quat,
    /// angular velocity information
    pub angular_velocity: Vec3,
}

impl TerseObjectData {
    pub fn from_bytes(data: &[u8]) -> Result<Self, ParseError> {
        let mut cursor = Cursor::new(data);

        let local_id = parse!(cursor.read_u32::<LittleEndian>())?;
        let state = parse!(cursor.read_u8())?;
        let avatar = parse!(cursor.read_u8())? != 0;

        let collision_plane = if avatar {
            let x = parse!(cursor.read_f32::<LittleEndian>())?;
            let y = parse!(cursor.read_f32::<LittleEndian>())?;
            let z = parse!(cursor.read_f32::<LittleEndian>())?;
            let w = parse!(cursor.read_f32::<LittleEndian>())?;

            Some(Vec4::new(x, y, z, w))
        } else {
            None
        };

        let position = Vec3::new(
            parse!(cursor.read_f32::<LittleEndian>())?,
            parse!(cursor.read_f32::<LittleEndian>())?,
            parse!(cursor.read_f32::<LittleEndian>())?,
        );

        let velocity = Vec3::new(
            parse!(u16_to_float_cursor(&mut cursor, -128.0, 128.0))?,
            parse!(u16_to_float_cursor(&mut cursor, -128.0, 128.0))?,
            parse!(u16_to_float_cursor(&mut cursor, -128.0, 128.0))?,
        );

        let acceleration = Vec3::new(
            parse!(u16_to_float_cursor(&mut cursor, -64.0, 64.0))?,
            parse!(u16_to_float_cursor(&mut cursor, -64.0, 64.0))?,
            parse!(u16_to_float_cursor(&mut cursor, -64.0, 64.0))?,
        );

        let rotation = Quat::from_xyzw(
            parse!(u16_to_float_cursor(&mut cursor, -1.0, 1.0))?,
            parse!(u16_to_float_cursor(&mut cursor, -1.0, 1.0))?,
            parse!(u16_to_float_cursor(&mut cursor, -1.0, 1.0))?,
            parse!(u16_to_float_cursor(&mut cursor, -1.0, 1.0))?,
        );

        let angular_velocity = Vec3::new(
            parse!(u16_to_float_cursor(&mut cursor, -64.0, 64.0))?,
            parse!(u16_to_float_cursor(&mut cursor, -64.0, 64.0))?,
            parse!(u16_to_float_cursor(&mut cursor, -64.0, 64.0))?,
        );

        Ok(Self {
            local_id,
            state,
            avatar,
            collision_plane,
            position,
            velocity,
            acceleration,
            rotation,
            angular_velocity,
        })
    }
}

fn u16_to_float_cursor(cursor: &mut Cursor<&[u8]>, min: f32, max: f32) -> Result<f32, ParseError> {
    let raw = parse!(cursor.read_u16::<LittleEndian>())?;
    Ok(min + (raw as f32) * ((max - min) / 65535.0))
}

impl PacketData for ImprovedTerseObjectUpdate {
    fn from_bytes(bytes: &[u8]) -> Result<Self, ParseError> {
        let mut cursor = Cursor::new(bytes);

        let region_handle = parse!(cursor.read_u64::<LittleEndian>())?;
        let time_dilation = parse!(cursor.read_u16::<LittleEndian>())?;

        let object_count = parse!(cursor.read_u8())?;
        let mut objects = Vec::with_capacity(object_count as usize);

        for _ in 0..object_count {
            let data_len = parse!(cursor.read_u8())?;

            let mut data_buf = vec![0; data_len as usize];
            parse!(cursor.read_exact(&mut data_buf))?;

            let data = parse!(TerseObjectData::from_bytes(&data_buf))?;
            objects.push(data);

            let tex_len = parse!(cursor.read_u16::<LittleEndian>())?;

            let mut tex_buf = vec![0; tex_len as usize];
            parse!(cursor.read_exact(&mut tex_buf))?;
        }

        Ok(Self {
            region_handle,
            time_dilation,
            objects,
        })
    }

    fn to_bytes(&self) -> Vec<u8> {
        Vec::new()
    }
}

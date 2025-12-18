use crate::errors::ParseError;
use crate::packet::{
    header::{Header, PacketFrequency},
    packet::{Packet, PacketData},
    packet_types::PacketType,
};
use crate::udp::object::object_update::MotionData;
use actix::Message;
use byteorder::{LittleEndian, ReadBytesExt};
use glam::{Quat, Vec3, Vec4};
use std::io::{Cursor, Read};
use uuid::Uuid;

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

#[derive(Debug, Message, Clone, Default)]
#[rtype(result = "()")]
pub struct ImprovedTerseObjectUpdate {
    pub region_handle: u64,
    pub time_dilation: u16,
    pub objects: Vec<TerseObjectData>,
}

#[derive(Debug, Clone)]
pub struct TerseObjectData {
    pub local_id: u32,
    pub state: u8,
    pub avatar: bool,
    pub collision_plane: Option<Vec4>,
    pub position: Vec3,
    pub velocity: Vec3,
    pub acceleration: Vec3,
    pub rotation: Quat,
    pub angular_velocity: Vec3,
}

impl TerseObjectData {
    pub fn from_bytes(data: &[u8]) -> Self {
        let mut cursor = Cursor::new(data);
        let local_id = cursor.read_u32::<LittleEndian>().unwrap();
        let state = cursor.read_u8().unwrap();
        let avatar = cursor.read_u8().unwrap() != 0;
        let collision_plane = if avatar {
            let x = cursor.read_f32::<LittleEndian>().unwrap();
            let y = cursor.read_f32::<LittleEndian>().unwrap();
            let z = cursor.read_f32::<LittleEndian>().unwrap();
            let w = cursor.read_f32::<LittleEndian>().unwrap();
            Some(Vec4::new(x, y, z, w))
        } else {
            None
        };
        let position = Vec3::new(
            cursor.read_f32::<LittleEndian>().unwrap(),
            cursor.read_f32::<LittleEndian>().unwrap(),
            cursor.read_f32::<LittleEndian>().unwrap(),
        );
        let velocity = Vec3::new(
            u16_to_float_cursor(&mut cursor, -128.0, 128.0),
            u16_to_float_cursor(&mut cursor, -128.0, 128.0),
            u16_to_float_cursor(&mut cursor, -128.0, 128.0),
        );
        let acceleration = Vec3::new(
            u16_to_float_cursor(&mut cursor, -64.0, 64.0),
            u16_to_float_cursor(&mut cursor, -64.0, 64.0),
            u16_to_float_cursor(&mut cursor, -64.0, 64.0),
        );
        let rotation = Quat::from_xyzw(
            u16_to_float_cursor(&mut cursor, -1.0, 1.0),
            u16_to_float_cursor(&mut cursor, -1.0, 1.0),
            u16_to_float_cursor(&mut cursor, -1.0, 1.0),
            u16_to_float_cursor(&mut cursor, -1.0, 1.0),
        );
        let angular_velocity = Vec3::new(
            u16_to_float_cursor(&mut cursor, -64.0, 64.0),
            u16_to_float_cursor(&mut cursor, -64.0, 64.0),
            u16_to_float_cursor(&mut cursor, -64.0, 64.0),
        );

        Self {
            local_id,
            state,
            avatar,
            collision_plane,
            position,
            velocity,
            acceleration,
            rotation,
            angular_velocity,
        }
    }
}

fn u16_to_float_cursor(cursor: &mut Cursor<&[u8]>, min: f32, max: f32) -> f32 {
    let raw = cursor.read_u16::<LittleEndian>().unwrap();
    min + (raw as f32) * ((max - min) / 65535.0)
}

impl PacketData for ImprovedTerseObjectUpdate {
    fn from_bytes(bytes: &[u8]) -> Result<Self, ParseError> {
        let mut cursor = Cursor::new(bytes);
        let region_handle = cursor.read_u64::<LittleEndian>()?;
        let time_dilation = cursor.read_u16::<LittleEndian>()?;

        let object_count = cursor.read_u8()?;
        let mut objects = Vec::with_capacity(object_count as usize);

        for _ in 0..object_count {
            let data_len = cursor.read_u8()?;
            let mut data_buf = vec![0; data_len as usize];
            cursor.read_exact(&mut data_buf)?;
            let data = TerseObjectData::from_bytes(&data_buf);
            objects.push(data);

            let tex_len = cursor.read_u16::<LittleEndian>()?;
            let mut tex_buf = vec![0; tex_len as usize];
            cursor.read_exact(&mut tex_buf)?;
        }

        Ok(ImprovedTerseObjectUpdate {
            region_handle,
            time_dilation,
            objects,
        })
    }
    fn to_bytes(&self) -> Vec<u8> {
        Vec::new()
    }
}

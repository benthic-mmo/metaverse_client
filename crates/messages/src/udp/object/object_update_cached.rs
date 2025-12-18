use actix::Message;
use byteorder::{LittleEndian, ReadBytesExt};

use crate::errors::ParseError;
use crate::packet::{
    header::{Header, PacketFrequency},
    packet::{Packet, PacketData},
    packet_types::PacketType,
};
use crate::udp::object::util::ObjectFlag;
use std::io::Cursor;

impl Packet {
    /// create a new object update cached packet
    pub fn new_object_update_cached(object_update_cached: ObjectUpdateCached) -> Self {
        Packet {
            header: Header {
                id: 14,
                reliable: true,
                zerocoded: false,
                frequency: PacketFrequency::High,
                ..Default::default()
            },
            body: PacketType::ObjectUpdateCached(Box::new(object_update_cached)),
        }
    }
}

#[derive(Debug, Message, Clone)]
#[rtype(result = "()")]
pub struct ObjectUpdateCached {
    pub region_handle: u64,
    pub time_dilation: u16,
    pub objects: Vec<CachedObjectData>,
}

#[derive(Debug, Clone)]
pub struct CachedObjectData {
    pub id: u32,
    pub crc: u32,
    pub flags: Vec<ObjectFlag>,
}

impl PacketData for ObjectUpdateCached {
    fn from_bytes(bytes: &[u8]) -> Result<Self, ParseError> {
        let mut cursor = Cursor::new(bytes);
        let region_handle = cursor.read_u64::<LittleEndian>()?;
        let time_dilation = cursor.read_u16::<LittleEndian>()?;
        let data_len = cursor.read_u8()?;

        let mut objects = Vec::new();
        for _ in 0..data_len {
            let id = cursor.read_u32::<LittleEndian>()?;
            let crc = cursor.read_u32::<LittleEndian>()?;
            let flags = ObjectFlag::from_bytes(cursor.read_u32::<LittleEndian>()?);
            objects.push(CachedObjectData { id, crc, flags });
        }
        Ok(ObjectUpdateCached {
            region_handle,
            time_dilation,
            objects,
        })
    }
    fn to_bytes(&self) -> Vec<u8> {
        Vec::new()
    }
}

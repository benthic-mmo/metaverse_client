use super::packet::PacketData;
use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use std::io::{self, Cursor};

#[derive(Debug)]
pub struct PacketAck {
    pub packet_ids: Vec<u32>,
}

impl PacketData for PacketAck {
    fn from_bytes(bytes: &[u8]) -> io::Result<Self> {
        let mut cursor = Cursor::new(bytes);

        let count = cursor.read_u8()? as usize;
        let mut packet_ids = Vec::with_capacity(count);

        for _ in 0..count {
            let id = cursor.read_u32::<LittleEndian>()?;
            packet_ids.push(id);
        }

        Ok(PacketAck { packet_ids })
    }

    fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();

        // Serialize packet IDs
        bytes.push(self.packet_ids.len() as u8);
        for id in &self.packet_ids {
            bytes.write_u32::<LittleEndian>(*id).unwrap();
        }

        bytes
    }
}

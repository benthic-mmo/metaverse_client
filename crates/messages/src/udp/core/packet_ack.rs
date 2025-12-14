use crate::{
    errors::ParseError,
    packet::{
        header::{Header, PacketFrequency},
        packet::{Packet, PacketData},
        packet_types::PacketType,
    },
};
use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use std::io::Cursor;

impl Packet {
    /// create a new acknowledgement packet
    pub fn new_packet_ack(packet_ack: PacketAck) -> Self {
        Packet {
            header: Header {
                id: 251,
                reliable: true,
                zerocoded: false,
                frequency: PacketFrequency::Fixed,
                ..Default::default()
            },
            body: PacketType::PacketAck(Box::new(packet_ack)),
        }
    }
}

#[derive(Debug, Clone)]
/// struct for ack packet. Contains packet IDs
pub struct PacketAck {
    /// list of IDs to ack
    pub packet_ids: Vec<u32>,
}

impl PacketData for PacketAck {
    fn from_bytes(bytes: &[u8]) -> Result<Self, ParseError> {
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

use super::{
    header::{Header, PacketFrequency},
    packet::{MessageType, Packet, PacketData},
};
use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use futures::future::BoxFuture;
use std::any::Any;
use std::{
    io::{self, Cursor},
    sync::Arc,
};

impl Packet {
    pub fn new_packet_ack(packet_ack: PacketAck) -> Self {
        Packet {
            header: Header {
                id: 251,
                reliable: false,
                resent: false,
                zerocoded: false,
                appended_acks: false,
                sequence_number: 0,
                frequency: PacketFrequency::Fixed,
                ack_list: None,
                size: None,
            },
            body: Arc::new(packet_ack),
        }
    }
}

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

    fn on_receive(&self) -> BoxFuture<'static, ()> {
        Box::pin(async move {
            println!("packet_ack on_receive is not yet implemented.");
        })
    }

    fn message_type(&self) -> MessageType {
        MessageType::Acknowledgment
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

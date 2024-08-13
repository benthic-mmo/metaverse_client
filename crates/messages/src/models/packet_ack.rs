use super::packet::{MessageType, PacketData};
use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use std::io::{self, Cursor};

// ID: 65531
// Frequency: Low

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

    fn on_receive(
        &self,
        ack_queue: std::sync::Arc<
            tokio::sync::Mutex<std::collections::HashMap<u32, tokio::sync::oneshot::Sender<()>>>,
        >,
    ) {
        let mut queue = futures::executor::block_on(ack_queue.lock());
        for id in &self.packet_ids {
            if let Some(sender) = queue.remove(&id) {
                let _ = sender.send(());
            } else {
                println!("No pending ack found for request ID: {}", id);
            }
        }
    }

    fn message_type(&self) -> MessageType {
        MessageType::Acknowledgment
    }
}

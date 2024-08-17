use super::{
    client_update_data::ClientUpdateData,
    packet::{MessageType, PacketData},
};
use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use futures::future::BoxFuture;
use std::{
    collections::HashMap,
    io::{self, Cursor},
    sync::Arc,
};
use tokio::sync::oneshot::Sender;
use std::sync::Mutex;

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
        ack_queue: Arc<Mutex<HashMap<u32, Sender<()>>>>,
        _: Arc<Mutex<Vec<ClientUpdateData>>>,
    ) -> BoxFuture<'static, ()> {
        let packet_ids = self.packet_ids.clone();

        Box::pin(async move {
            let mut queue = ack_queue.lock().unwrap();
            for id in packet_ids {
                if let Some(sender) = queue.remove(&id) {
                    let _ = sender.send(());
                } else {
                    println!("No pending ack found for request ID: {}", id);
                }
            }
        })
    }

    fn message_type(&self) -> MessageType {
        MessageType::Acknowledgment
    }
}

use crate::models::header::{Header, PacketFrequency};
use crate::models::packet::{Packet, PacketData};
use futures::future::BoxFuture;
use std::any::Any;
use std::collections::HashMap;
use std::io;
use std::sync::Arc;
use std::sync::Mutex;
use tokio::sync::oneshot::Sender;
use uuid::Uuid;

use super::client_update_data::ClientUpdateData;
use super::packet::MessageType;

impl Packet {
    pub fn new_circuit_code(circuit_code_block: CircuitCodeData) -> Self {
        Packet {
            header: Header {
                id: 3,
                frequency: PacketFrequency::Low,
                reliable: true,
                sequence_number: 0,
                appended_acks: false,
                zerocoded: false,
                resent: false,
                ack_list: None,
                size: None,
            },
            body: Arc::new(circuit_code_block),
        }
    }
}

#[derive(Debug)]
pub struct CircuitCodeData {
    pub code: u32,
    pub session_id: Uuid,
    pub id: Uuid,
}

impl PacketData for CircuitCodeData {
    fn from_bytes(bytes: &[u8]) -> io::Result<Self> {
        let code = u32::from_le_bytes(bytes[0..4].try_into().unwrap());
        let session_id = Uuid::from_slice(&bytes[4..20]).unwrap();
        let id = Uuid::from_slice(&bytes[20..36]).unwrap();

        Ok(Self {
            code,
            session_id,
            id,
        })
    }
    fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(36);
        bytes.extend_from_slice(&self.code.to_le_bytes());
        bytes.extend(self.session_id.as_bytes());
        bytes.extend(self.id.as_bytes());
        bytes
    }
    fn on_receive(
        &self,
        _: Arc<Mutex<HashMap<u32, Sender<()>>>>,
        _: Arc<Mutex<Vec<ClientUpdateData>>>,
    ) -> BoxFuture<'static, ()> {
        Box::pin(async move {
            // Implement the actual logic here later
            println!("on_receive is not yet implemented.");
        })
    }
    fn message_type(&self) -> MessageType {
        MessageType::Outgoing
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

use std::sync::Arc;

use uuid::Uuid;

use super::{
    header::{Header, PacketFrequency},
    packet::{MessageType, Packet, PacketData},
};

impl Packet {
    pub fn new_complete_agent_movement(
        complete_agent_movement_data: CompleteAgentMovementData,
    ) -> Self {
        Packet {
            header: Header {
                id: 249,
                frequency: PacketFrequency::Low,
                reliable: false,
                sequence_number: 3,
                appended_acks: false,
                zerocoded: false,
                resent: false,
                ack_list: None,
                size: None,
            },
            body: Arc::new(complete_agent_movement_data),
        }
    }
}

#[derive(Debug)]
pub struct CompleteAgentMovementData {
    pub agent_id: Uuid,
    pub session_id: Uuid,
    pub circuit_code: u32,
}

impl PacketData for CompleteAgentMovementData {
    fn from_bytes(bytes: &[u8]) -> std::io::Result<Self> {
        let circuit_code = u32::from_le_bytes(bytes[0..4].try_into().unwrap());
        let session_id = Uuid::from_slice(&bytes[4..20]).unwrap();
        let agent_id = Uuid::from_slice(&bytes[20..36]).unwrap();

        Ok(CompleteAgentMovementData {
            agent_id,
            session_id,
            circuit_code,
        })
    }
    fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(36);
        bytes.extend_from_slice(self.agent_id.as_bytes());
        bytes.extend(self.session_id.as_bytes());
        bytes.extend(self.circuit_code.to_le_bytes());
        bytes
    }
    fn on_receive(
        &self,
        queue: Arc<
            tokio::sync::Mutex<std::collections::HashMap<u32, tokio::sync::oneshot::Sender<()>>>,
        >,
    ) -> futures::future::BoxFuture<'static, ()> {
        Box::pin(async move {
            // Implement the actual logic here later
            println!("on_receive is not yet implemented.");
        })
    }
    fn message_type(&self) -> MessageType {
        MessageType::Outgoing
    }
}

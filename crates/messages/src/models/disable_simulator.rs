use super::{
    client_update_data::ClientUpdateData,
    packet::{MessageType, PacketData},
};
use futures::future::BoxFuture;
use std::sync::Mutex;
use std::{collections::HashMap, io, sync::Arc};
use tokio::sync::oneshot::Sender;

// ID: 152
// Frequency: Low

#[derive(Debug)]
pub struct DisableSimulator {}

impl PacketData for DisableSimulator {
    fn from_bytes(_: &[u8]) -> io::Result<Self> {
        Ok(DisableSimulator {})
    }
    fn to_bytes(&self) -> Vec<u8> {
        vec![]
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
        MessageType::Event
    }
}

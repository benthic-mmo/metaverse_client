use super::packet::{MessageType, PacketData};
use futures::future::BoxFuture;
use std::io;

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
        _: std::sync::Arc<
            tokio::sync::Mutex<std::collections::HashMap<u32, tokio::sync::oneshot::Sender<()>>>,
        >,
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

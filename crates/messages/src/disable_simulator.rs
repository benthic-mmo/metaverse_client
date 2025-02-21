use super::packet::{MessageType, PacketData};
use futures::future::BoxFuture;
use std::any::Any;
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
    fn on_receive(&self) -> BoxFuture<'static, ()> {
        Box::pin(async move {
            println!("disable_simulator on_receive is not yet implemented.");
        })
    }
    fn message_type(&self) -> MessageType {
        MessageType::Event
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

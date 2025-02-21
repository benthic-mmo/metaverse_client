use super::packet::PacketData;
use futures::future::BoxFuture;
use std::io;

// ID: 152
// Frequency: Low

#[derive(Debug, Clone)]
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
}

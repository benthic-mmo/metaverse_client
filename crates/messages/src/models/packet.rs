use crate::models::header::Header;
use actix::prelude::*;
use futures::future::BoxFuture;
use std::collections::HashMap;
use std::io;
use std::sync::Arc;
use std::sync::Mutex;
use tokio::sync::oneshot::Sender;

use super::client_update_data::ClientUpdateData;
use super::packet_types::PacketType;

#[derive(Debug, Message, Clone)]
#[rtype(result = "()")]
pub struct Packet {
    pub header: Header,
    pub body: Arc<dyn PacketData>,
}

pub enum MessageType {
    Acknowledgment,
    Request,
    Event,
    Command,
    Error,
    Data,
    Outgoing,
}

// this is the trait that allows for serializing and deserializing the packet's data
pub trait PacketData: std::fmt::Debug + Send + Sync + 'static {
    fn from_bytes(bytes: &[u8]) -> io::Result<Self>
    where
        Self: Sized;
    fn to_bytes(&self) -> Vec<u8>;
    fn on_receive(
        &self,
        queue: Arc<Mutex<HashMap<u32, Sender<()>>>>,
        update_stream: Arc<Mutex<Vec<ClientUpdateData>>>,
    ) -> BoxFuture<'static, ()>;
    fn message_type(&self) -> MessageType;
}

impl Packet {
    pub fn from_bytes(bytes: &[u8]) -> io::Result<Self> {
        let header = Header::try_from_bytes(bytes).unwrap();
        // if the packet has a body, add the body to the packet
        let body_bytes = if header.size.unwrap_or(0) < bytes.len() {
            &bytes[header.size.unwrap_or(0)..]
        } else {
            &[]
        };

        let body = PacketType::from_id(header.id, header.frequency, body_bytes)?.into_arc();
        Ok(Self { header, body })
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend(self.header.to_bytes());
        bytes.extend(self.body.to_bytes());
        bytes
    }
}

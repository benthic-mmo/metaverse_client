use std::error::Error;
use std::sync::Arc;

use std::sync::Mutex;

use super::chat_from_simulator::ChatFromSimulator;
use super::packet::Packet;

#[derive(Debug)]
pub enum ClientUpdateData {
    String(String),
    Packet(Packet),
    LoginProgress(LoginProgress),
    Error(Box<dyn Error + Send + Sync>),
    ChatFromSimulator(ChatFromSimulator),
}

#[derive(Debug)]
pub struct LoginProgress {
    pub message: String,
    pub percent: u8,
}

// Implement the From traits to make conversion easy
impl From<String> for ClientUpdateData {
    fn from(value: String) -> Self {
        ClientUpdateData::String(value)
    }
}

impl From<Packet> for ClientUpdateData {
    fn from(value: Packet) -> Self {
        ClientUpdateData::Packet(value)
    }
}

impl From<LoginProgress> for ClientUpdateData {
    fn from(value: LoginProgress) -> Self {
        ClientUpdateData::LoginProgress(value)
    }
}

impl From<Box<dyn Error + Send + Sync>> for ClientUpdateData {
    fn from(value: Box<dyn Error + Send + Sync>) -> Self {
        ClientUpdateData::Error(value)
    }
}

impl From<ChatFromSimulator> for ClientUpdateData {
    fn from(value: ChatFromSimulator) -> Self {
        ClientUpdateData::ChatFromSimulator(value)
    }
}
pub async fn send_message_to_client(
    stream: Arc<Mutex<Vec<ClientUpdateData>>>,
    content: ClientUpdateData,
) {
    let mut stream = stream.lock().unwrap();
    stream.push(content);
}

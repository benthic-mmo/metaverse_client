use super::{
    client_update_data::{send_message_to_client, ClientUpdateData},
    header::{Header, PacketFrequency},
    packet::{MessageType, Packet, PacketData},
};
use byteorder::{BigEndian, LittleEndian, ReadBytesExt};
use futures::future::BoxFuture;
use std::{
    any::Any,
    collections::HashMap,
    io::{self, Cursor},
    sync::Arc,
};
use std::{io::Read, sync::Mutex};
use tokio::sync::oneshot::Sender;
use uuid::Uuid;

impl Packet {
    pub fn new_chat_from_viewer(chat_from_viewer: ChatFromViewer) -> Self {
        Packet {
            header: Header {
                id: 80,
                frequency: PacketFrequency::Low,
                reliable: true,
                sequence_number: 0,
                appended_acks: false,
                zerocoded: false,
                resent: false,
                ack_list: None,
                size: None,
            },
            body: Arc::new(chat_from_viewer),
        }
    }
}

#[derive(Debug)]
pub struct ChatFromViewer {
    pub agent_id: Uuid,
    pub session_id: Uuid,
    pub message: String,
    pub message_type: ClientChatType,
    pub channel: i32,
}

#[derive(Debug)]
pub enum ClientChatType {
    Whisper,
    Normal,
    Shout,
    Say,
    StartTyping,
    StopTyping,
    Debug,
    Unknown,
}

impl ClientChatType {
    pub fn to_bytes(&self) -> u8 {
        match self {
            ClientChatType::Whisper => 0,
            ClientChatType::Normal => 1,
            ClientChatType::Shout => 2,
            ClientChatType::Say => 3,
            ClientChatType::StartTyping => 4,
            ClientChatType::StopTyping => 5,
            ClientChatType::Debug => 6,
            ClientChatType::Unknown => 7,
        }
    }
    pub fn from_bytes(bytes: u8) -> Self {
        match bytes {
            0 => ClientChatType::Whisper,
            1 => ClientChatType::Normal,
            2 => ClientChatType::Shout,
            3 => ClientChatType::Say,
            4 => ClientChatType::StartTyping,
            5 => ClientChatType::StopTyping,
            6 => ClientChatType::Debug,
            _ => ClientChatType::Unknown,
        }
    }
}

impl PacketData for ChatFromViewer {
    fn from_bytes(bytes: &[u8]) -> io::Result<Self> {
        let mut cursor = Cursor::new(bytes);

        // Deserialize AgentData
        let mut agent_id_bytes = [0u8; 16];
        cursor.read_exact(&mut agent_id_bytes)?;
        let agent_id = Uuid::from_bytes(agent_id_bytes);

        let mut session_id_bytes = [0u8; 16];
        cursor.read_exact(&mut session_id_bytes)?;
        let session_id = Uuid::from_bytes(session_id_bytes);

        let message_length = cursor.read_u16::<LittleEndian>()? as usize;

        let mut message_bytes = vec![0u8; message_length];
        cursor.read_exact(&mut message_bytes)?;

        let message = String::from_utf8(message_bytes).unwrap();

        let message_type_byte = cursor.read_u8()?;
        let message_type = ClientChatType::from_bytes(message_type_byte);

        let channel = cursor.read_i32::<BigEndian>()?;

        Ok(ChatFromViewer {
            agent_id,
            session_id,
            message,
            message_type,
            channel,
        })
    }

    fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();

        bytes.extend_from_slice(self.agent_id.as_bytes());
        bytes.extend_from_slice(self.session_id.as_bytes());

        let message_bytes = self.message.as_bytes();

        bytes.extend_from_slice(&(message_bytes.len() as u16).to_le_bytes());
        bytes.extend_from_slice(message_bytes);

        bytes.push(self.message_type.to_bytes());
        bytes.extend_from_slice(&self.channel.to_le_bytes());

        bytes.push(0);

        bytes
    }

    fn on_receive(
        &self,
        _: Arc<Mutex<HashMap<u32, Sender<()>>>>,
        client_update: Arc<Mutex<Vec<ClientUpdateData>>>,
    ) -> BoxFuture<'static, ()> {
        //let chat_data: ClientUpdateData = ClientUpdateData::ChatFromSimulator(self.clone());
        let message_clone = self.message.clone();
        Box::pin(async move {
            send_message_to_client(
                client_update.clone(),
                ClientUpdateData::String(
                    format!("sent message!!!!!! {}", message_clone).to_string(),
                ),
            )
            .await
        })
    }

    fn message_type(&self) -> MessageType {
        MessageType::Event
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

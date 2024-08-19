use super::{
    agent_update::Vector3,
    client_update_data::{send_message_to_client, ClientUpdateData},
    header::Header,
    packet::{MessageType, Packet, PacketData},
};
use byteorder::ReadBytesExt;
use futures::future::BoxFuture;
use std::{
    collections::HashMap,
    io::{self, BufRead, Cursor},
    sync::Arc,
};
use std::{io::Read, sync::Mutex};
use tokio::sync::oneshot::Sender;
use uuid::Uuid;

// ID: 139
// Frequency: Low

impl Packet {
    pub fn new_chat_from_simulator(
        chat_from_simulator: ChatFromSimulator,
    ) -> Self {
        Packet {
            header: Header {
                id: 139,
                frequency: super::header::PacketFrequency::Low,
                reliable: false,
                sequence_number: 0,
                appended_acks: false,
                zerocoded: false,
                resent: false,
                ack_list: None,
                size: None,
            },
            body: Arc::new(chat_from_simulator),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ChatFromSimulator {
    pub from_name: String,
    pub source_id: Uuid,
    pub owner_id: Uuid,
    pub source_type: SourceType,
    pub chat_type: ChatType,
    pub audible: Audible,
    pub position: Vector3,
    pub message: String,
}

#[derive(Debug, Clone)]
pub enum SourceType {
    System,
    Agent,
    Object,
    Unknown,
}
impl SourceType {
    fn from_bytes(bytes: u8) -> Self {
        match bytes {
            0 => SourceType::System,
            1 => SourceType::Agent,
            2 => SourceType::Object,
            _ => SourceType::Unknown,
        }
    }
    fn to_bytes(&self) -> u8 {
        match self {
            SourceType::System => 0,
            SourceType::Agent => 1,
            SourceType::Object => 2,
            SourceType::Unknown => 3,
        }
    }
}

#[derive(Debug, Clone)]
pub enum Audible {
    Not,
    Barely,
    Fully,
    Unknown,
}
impl Audible {
    fn from_bytes(bytes: u8) -> Self {
        match bytes {
            255 => Audible::Not,
            0 => Audible::Barely,
            1 => Audible::Fully,
            _ => Audible::Unknown,
        }
    }
    fn to_bytes(&self) -> u8 {
        match self {
            Audible::Not => 255,
            Audible::Barely => 0,
            Audible::Fully => 1,
            Audible::Unknown => 2,
        }
    }
}

#[derive(Debug, Clone)]
pub enum ChatType {
    Whisper,
    Normal,
    Shout,
    Say,
    StartTyping,
    StopTyping,
    Debug,
    OwnerSay,
    Unknown,
}
impl ChatType {
    pub fn from_bytes(bytes: u8) -> Self {
        match bytes {
            0 => ChatType::Whisper,
            1 => ChatType::Normal,
            2 => ChatType::Shout,
            3 => ChatType::Say,
            4 => ChatType::StartTyping,
            5 => ChatType::StopTyping,
            6 => ChatType::Debug,
            8 => ChatType::OwnerSay,
            _ => ChatType::Unknown,
        }
    }
    pub fn to_bytes(&self) -> u8 {
        match self {
            ChatType::Whisper => 0,
            ChatType::Normal => 1,
            ChatType::Shout => 2,
            ChatType::Say => 3,
            ChatType::StartTyping => 4,
            ChatType::StopTyping => 5,
            ChatType::Debug => 6,
            ChatType::OwnerSay => 8,
            ChatType::Unknown => 9,
        }
    }
}

impl PacketData for ChatFromSimulator {
    fn from_bytes(bytes: &[u8]) -> io::Result<Self> {
        let mut cursor = Cursor::new(bytes);

        // FromName
        let mut from_name_bytes = Vec::new();
        cursor.read_until(0, &mut from_name_bytes)?; // Read until null terminator
        let from_name =
            String::from_utf8(from_name_bytes.into_iter().filter(|&b| b != 0).collect())
                .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

        // SourceID
        let source_id = Uuid::from_slice(
            &cursor.get_ref()[cursor.position() as usize..cursor.position() as usize + 16],
        )
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        cursor.set_position(cursor.position() + 16);

        // OwnerID
        let owner_id = Uuid::from_slice(
            &cursor.get_ref()[cursor.position() as usize..cursor.position() as usize + 16],
        )
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        cursor.set_position(cursor.position() + 16);

        // SourceType
        let source_type_byte = cursor.read_u8()?;
        let source_type = SourceType::from_bytes(source_type_byte);

        // ChatType
        let chat_type_byte = cursor.read_u8()?;
        let chat_type = ChatType::from_bytes(chat_type_byte);

        // Audible
        let audible_byte = cursor.read_u8()?;
        let audible = Audible::from_bytes(audible_byte);

        // Position (LLVector3)
        let position = Vector3 {
            x: cursor.read_f32::<byteorder::LittleEndian>()?,
            y: cursor.read_f32::<byteorder::LittleEndian>()?,
            z: cursor.read_f32::<byteorder::LittleEndian>()?,
        };

        // Message
        let mut message_bytes = Vec::new();
        cursor.read_to_end(&mut message_bytes)?;
        let message = String::from_utf8(message_bytes)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

        Ok(Self {
            from_name,
            source_id,
            owner_id,
            source_type,
            chat_type,
            audible,
            position,
            message,
        })
    }

    fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();

        // Convert `from_name` to bytes (length-prefixed)
        let name_bytes = self.from_name.as_bytes();
        bytes.extend_from_slice(&(name_bytes.len() as u32).to_le_bytes());
        bytes.extend_from_slice(name_bytes);

        // Convert `source_id` and `owner_id` to bytes
        bytes.extend_from_slice(self.source_id.as_bytes());
        bytes.extend_from_slice(self.owner_id.as_bytes());

        // Convert `source_type`, `chat_type`, and `audible` to bytes
        bytes.push(self.source_type.to_bytes());
        bytes.push(self.chat_type.to_bytes());
        bytes.push(self.audible.to_bytes());

        // Convert `position` (Vector3<f32>) to bytes
        bytes.extend_from_slice(&self.position.x.to_le_bytes());
        bytes.extend_from_slice(&self.position.y.to_le_bytes());
        bytes.extend_from_slice(&self.position.z.to_le_bytes());

        // Convert `message` to bytes (length-prefixed)
        let message_bytes = self.message.as_bytes();
        bytes.extend_from_slice(&(message_bytes.len() as u32).to_le_bytes());
        bytes.extend_from_slice(message_bytes);

        bytes
    }

    fn on_receive(
        &self,
        _: Arc<Mutex<HashMap<u32, Sender<()>>>>,
        client_update: Arc<Mutex<Vec<ClientUpdateData>>>,
    ) -> BoxFuture<'static, ()> {
        let chat_data: ClientUpdateData = ClientUpdateData::ChatFromSimulator(self.clone());

        Box::pin(async move { send_message_to_client(client_update.clone(), chat_data).await })
    }

    fn message_type(&self) -> MessageType {
        MessageType::Outgoing
    }
}

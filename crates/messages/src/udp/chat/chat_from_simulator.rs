use crate::packet::{
    errors::PacketError,
    header::{Header, PacketFrequency},
    message::EventType,
    packet::{Packet, PacketData},
    packet_types::PacketType,
};
use byteorder::ReadBytesExt;
use glam::Vec3;
use serde::{Deserialize, Serialize};
use std::io::Read;
use std::io::{self, BufRead, Cursor};
use uuid::Uuid;

use super::ChatType;

impl Packet {
    /// Create a new chat from simulator packet
    pub fn new_chat_from_simulator(chat_from_simulator: ChatFromSimulator) -> Self {
        Packet {
            header: Header {
                id: 139,
                frequency: PacketFrequency::Low,
                reliable: true,
                sequence_number: 0,
                appended_acks: false,
                zerocoded: false,
                resent: false,
                ack_list: None,
                size: None,
            },
            body: PacketType::ChatFromSimulator(Box::new(chat_from_simulator)),
        }
    }
}

/// Implement UIEvent for ChatFromSimulator. Can be sent between the core and UI
impl EventType {
    /// create a new chat from simulator UiEvent
    pub fn new_chat_from_simulator(data: ChatFromSimulator) -> Self {
        EventType::ChatFromSimulator(data)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
/// Struct for notifying client of new chat messages
pub struct ChatFromSimulator {
    /// The person who sent the message
    pub from_name: String,
    /// the UUID of the source of the message
    pub source_id: Uuid,
    /// undocumented
    pub owner_id: Uuid,
    /// The type of agent that emitted the chat message
    pub source_type: SourceType,
    /// The type of chat, whisper, speak, shout, etc
    pub chat_type: ChatType,
    /// If the chat is audible to the user or not
    pub audible: Audible,
    /// position of the chat. Currently unused.
    pub position: Vec3,
    /// The contents of the message
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
/// Enum for the types of agents that can emit chat messages
pub enum SourceType {
    /// chat coming from the system
    System,
    /// chat coming from another user
    Agent,
    /// chat coming from an object
    Object,
    /// chat coming from an unknown source
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

#[derive(Debug, Clone, Serialize, Deserialize)]
/// Determines if the chat is audible to the user.
pub enum Audible {
    /// Not audible. Don't display the chat.
    Not,
    /// The chat is faint
    Barely,
    /// The chat is fully audible
    Fully,
    /// Unknown
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

impl PacketData for ChatFromSimulator {
    fn from_bytes(bytes: &[u8]) -> Result<Self, PacketError> {
        let mut cursor = Cursor::new(bytes);

        // FromName
        // skip one byte of prefix, for some reason
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
        let position = Vec3 {
            x: cursor.read_f32::<byteorder::LittleEndian>()?,
            y: cursor.read_f32::<byteorder::LittleEndian>()?,
            z: cursor.read_f32::<byteorder::LittleEndian>()?,
        };

        // skip two bytes of size prefix
        cursor.set_position(cursor.position() + 1);
        // Message
        let mut message_bytes = Vec::new();
        cursor.read_to_end(&mut message_bytes)?;
        if !message_bytes.is_empty() {
            message_bytes.pop();
        }
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

        // Convert `from_name` to bytes (null-terminated)
        let name_bytes = self.from_name.as_bytes();
        bytes.extend_from_slice(name_bytes);
        bytes.push(0);

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

        // Convert `message` to bytes (null-terminated)
        let message_bytes = self.message.as_bytes();
        bytes.extend_from_slice(message_bytes);
        bytes.push(0);

        bytes
    }
}

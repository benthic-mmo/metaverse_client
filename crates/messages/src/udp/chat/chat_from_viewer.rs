use crate::{
    errors::ParseError,
    packet::{
        header::{Header, PacketFrequency},
        packet::{Packet, PacketData},
        packet_types::PacketType,
    },
};
use byteorder::{BigEndian, LittleEndian, ReadBytesExt};
use serde::{Deserialize, Serialize};
use std::io::Cursor;
use std::io::Read;
use uuid::Uuid;

use super::ChatType;

impl Packet {
    /// Create a new chat from viewer packet
    pub fn new_chat_from_viewer(chat_from_viewer: ChatFromViewer) -> Self {
        Packet {
            header: Header {
                id: 80,
                frequency: PacketFrequency::Low,
                reliable: false,
                zerocoded: false,
                ..Default::default()
            },
            body: PacketType::ChatFromViewer(Box::new(chat_from_viewer)),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
/// ChatFromViewer struct. Contains information about chat messages sent from the viewer to the
/// server.
pub struct ChatFromViewer {
    /// The ID of the agent who sent the chat
    pub agent_id: Uuid,
    /// the ID of the session who sent the chat
    pub session_id: Uuid,
    /// the message of the chat
    pub message: String,
    /// the type of the chat
    pub message_type: ChatType,
    /// the channel the message was sent on
    pub channel: i32,
}

impl PacketData for ChatFromViewer {
    fn from_bytes(bytes: &[u8]) -> Result<Self, ParseError> {
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

        let message = String::from_utf8(message_bytes)?;

        let message_type_byte = cursor.read_u8()?;
        let message_type = ChatType::from_bytes(message_type_byte);

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
}

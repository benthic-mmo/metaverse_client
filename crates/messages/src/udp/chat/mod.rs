use serde::{Deserialize, Serialize};

/// # ChatFromSimulator
/// <https://wiki.secondlife.com/wiki/ChatFromSimulator>
///
/// The chat packet sent from the simulator to the viewer. Contains data about chat messages being
/// sent in the world.
///
/// ## Header
/// | ChatFromSimulator |||||
/// |--------------|---------------|----------------|-------------------|---------------------|
/// | Packet Header| id:139        | reliable: false| zerocoded: false  | frequency: Low      |
///
/// ## Packet Structure
/// | ChatData      |         |             |                                                     |
/// |---------------|---------|-------------|-----------------------------------------------------|
/// | FromName      |variable bytes (null terminated) | [String] | Name of user that sent the chat|
/// | SourceID      |16 bytes | [Uuid](uuid::Uuid)    | Agent ID of the user who sent the chat    |
/// | OwnerID       |16 bytes | [Uuid](uuid::Uuid)    | Undocumented                              |
/// | SourceType    |1 byte   | [u8]| The type of thing that emitted the chat. (system, agent or object)|
/// | ChatType      |1 byte   | [u8]| type of chat, like "say", "whisper" and "yell"              |
/// | Audible       |1 byte   | [u8]| If the message is audible or not                            |
/// | Position      |12 bytes | [Vector3](glam::Vec3)[[u8]] | The position of the message. Unused |
/// | Message       |variable bytes (null terminated) | [String] | Contents of chat message       |
pub mod chat_from_simulator;

/// # ChatFromViewer
/// <https://wiki.secondlife.com/wiki/ChatFromViewer>
///
/// The chat packet sent from the viewer to the server
///
/// ## Header
/// | ChatFromViewer |||||
/// |--------------|---------------|----------------|-------------------|---------------------|
/// | Packet Header| id:80        | reliable: true  | zerocoded: false  | frequency: Low      |
///
/// ## Packet Structure
/// | ChatData      |          |             |                                                     |
/// |---------------|----------|-------------|-----------------------------------------------------|
/// | AgentID       | 16 bytes | [Uuid](uuid::Uuid)| the ID of your agent                          |
/// | SessionID     | 16 bytes | [Uuid](uuid::Uuid)| The ID of your session                        |
/// | Message       | variable bytes (null terminated) | [String] | Chat message sent from user    |
/// | Type          | 1 byte   | [u8]  | Type of chat, like "say", "whisper" and "yell"            |
/// | Channel       | 4 bytes  | [i32] | Channel to send the message on.                           |
pub mod chat_from_viewer;

#[derive(Debug, Clone, Serialize, Deserialize)]
/// The type of chats that can be sent
pub enum ChatType {
    /// A chat spoken at a whisper
    Whisper,
    /// A chat spoken at normal volume
    Normal,
    /// A chat shouted
    Shout,
    /// A chat said
    Say,
    /// Indicates typing has started
    StartTyping,
    /// indicates typing has stopped
    StopTyping,
    /// a debug chat
    Debug,
    /// an owner announcement
    OwnerSay,
    /// unknown
    Unknown,
}
impl ChatType {
    /// parse the chat type from the packet bytes
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
    /// convert the chat type to bytes for sending packets
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

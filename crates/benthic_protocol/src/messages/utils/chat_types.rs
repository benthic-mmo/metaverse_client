use serde::{Deserialize, Serialize};

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
    pub fn from_bytes(bytes: u8) -> Self {
        match bytes {
            0 => SourceType::System,
            1 => SourceType::Agent,
            2 => SourceType::Object,
            _ => SourceType::Unknown,
        }
    }
    pub fn to_bytes(&self) -> u8 {
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
    pub fn from_bytes(bytes: u8) -> Self {
        match bytes {
            255 => Audible::Not,
            0 => Audible::Barely,
            1 => Audible::Fully,
            _ => Audible::Unknown,
        }
    }
    pub fn to_bytes(&self) -> u8 {
        match self {
            Audible::Not => 255,
            Audible::Barely => 0,
            Audible::Fully => 1,
            Audible::Unknown => 2,
        }
    }
}

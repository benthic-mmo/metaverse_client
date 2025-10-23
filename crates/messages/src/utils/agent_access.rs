use serde::{Deserialize, Serialize};
use serde_llsd::{LLSDValue, converter::FromLLSDValue};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
/// Handles the access level of the viewer.
/// prevents people with lower access levels from joining regions they shouldn't.
pub enum AgentAccess {
    /// Agent can view adult, 18+ content
    Adult,
    /// Agent can view mature, 18+ content
    Mature,
    /// undocumented
    Down,
    /// undocumented
    NonExistent,
    /// trial account
    Trial,
    /// Agent can view general audiences content
    General,
    /// Agent can view PG rated content
    PG,
    /// unknown
    Unknown,
}
impl FromLLSDValue for AgentAccess {
    fn from_llsd(value: &LLSDValue) -> Option<Self> {
        if let LLSDValue::String(s) = value {
            match s.as_str() {
                "A" => Some(AgentAccess::Adult),
                "M" => Some(AgentAccess::Mature),
                "D" => Some(AgentAccess::Down),
                "N" => Some(AgentAccess::NonExistent),
                "T" => Some(AgentAccess::Trial),
                "G" => Some(AgentAccess::General),
                "PG" => Some(AgentAccess::PG),
                _ => Some(AgentAccess::Unknown),
            }
        } else {
            None
        }
    }
}
impl AgentAccess {
    /// Maps agent access values to their byte representation. These are a little randomly chosen.
    pub fn to_bytes(&self) -> u8 {
        match self {
            AgentAccess::General => 2,
            AgentAccess::Trial => 7,
            AgentAccess::PG => 13,
            AgentAccess::Mature => 21,
            AgentAccess::Adult => 42,
            AgentAccess::Down => 254,
            AgentAccess::NonExistent => 255,
            _ => 0,
        }
    }
    /// Convert from byte representation to enum
    pub fn from_bytes(bytes: &u8) -> Self {
        match bytes {
            2 => AgentAccess::General,
            7 => AgentAccess::Trial,
            13 => AgentAccess::PG,
            21 => AgentAccess::Mature,
            42 => AgentAccess::Adult,
            254 => AgentAccess::Down,
            255 => AgentAccess::NonExistent,
            _ => AgentAccess::Unknown,
        }
    }
}

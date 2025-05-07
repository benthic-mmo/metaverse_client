use serde::{Deserialize, Serialize};
use xmlrpc_benthic::{self as xmlrpc, Value};

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
impl From<AgentAccess> for Value {
    fn from(val: AgentAccess) -> Self {
        let access_str = match val {
            AgentAccess::Down => "Down",
            AgentAccess::NonExistent => "",
            AgentAccess::Trial => "T",
            AgentAccess::Mature => "M",
            AgentAccess::Adult => "A",
            AgentAccess::PG => "PG",
            AgentAccess::General => "G",
            AgentAccess::Unknown => "Unknown",
        };
        Value::String(access_str.to_string())
    }
}

/// used by the login response to parse the agent access from a string.
pub fn parse_agent_access(agent_access: Option<&xmlrpc::Value>) -> Option<AgentAccess> {
    agent_access.map(|x| match x.clone().as_str().unwrap() {
        "M" => AgentAccess::Mature,
        "A" => AgentAccess::Adult,
        "PG" => AgentAccess::PG,
        "G" => AgentAccess::General,
        "" => AgentAccess::NonExistent,
        "Down" => AgentAccess::Down,
        "T" => AgentAccess::Trial,
        _ => AgentAccess::Unknown,
    })
}

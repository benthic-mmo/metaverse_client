use core::fmt;
use std::{collections::HashMap, fmt::Display};

use actix::Message;
use serde_llsd::LLSDValue;

use crate::errors::ParseError;

#[derive(Debug, Clone, Hash, Eq, PartialEq)]
/// Describes the capabilities the client can have.
/// these are sent to the server to retrieve the URL of the capability endpoint.
/// that URL can be used to retrieve more data from the server.
///
/// There are many other legacy capabilities, which will not be implmented. Only the ones that
/// currently work will be added here.
pub enum Capability {
    /// Enable the viewer to retrieve assets from the asset server.  
    ViewerAsset,
    /// Enable the viewer to retrieve the inventory of the current user. Required for determining
    /// the user's appearance and managing inventory.
    FetchInventoryDescendents2,
    /// Enable the viewer to retrieve the library inventory. This is the public library which
    /// contains data about other people's objects. Required for determining other user's
    /// appearances.
    FetchLibDescendents2,
    /// Unknown
    Unknown,
}
impl Capability {
    fn from_string(string: &str) -> Self {
        match string {
            "ViewerAsset" => Self::ViewerAsset,
            "FetchLibDescendents2" => Self::FetchLibDescendents2,
            "FetchInventoryDescendents2" => Self::FetchInventoryDescendents2,
            _ => Self::Unknown,
        }
    }
}
impl Display for Capability {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ViewerAsset => write!(f, "ViewerAsset"),
            Self::FetchInventoryDescendents2 => write!(f, "FetchInventoryDescendents2"),
            Self::FetchLibDescendents2 => write!(f, "FetchLibDescendents2"),
            Self::Unknown => write!(f, "Unknown"),
        }
    }
}

#[derive(Debug, Message, Clone)]
#[rtype(result = "()")]
/// This is the type that is sent to the actix session to handle sending capability requests.
pub struct CapabilityRequest {
    /// This is an xml string of the capabilities the client wants.
    pub capabilities: String,
}

impl CapabilityRequest {
    /// generate the XML for creating a new capability request.
    /// accepts a vector of capabilities you want enabled.
    /// this will be sent to the server's seed capability endpoint (which is received by the login
    /// response), and will return the requested endpoint URLs.
    pub fn new_capability_request(capabilities: Vec<Capability>) -> Result<Self, ParseError> {
        let mut capability_vec = Vec::new();
        for capability in capabilities {
            capability_vec.push(LLSDValue::String(capability.to_string()));
        }
        let caps = serde_llsd::ser::xml::to_string(&LLSDValue::Array(capability_vec), false)?;

        Ok(CapabilityRequest { capabilities: caps })
    }
    /// Generate the HashMap from the response bytes. This should be stored and used by the session
    /// to retrieve information from the requested endpoints.
    pub fn response_from_llsd(xml_bytes: &[u8]) -> Result<HashMap<Capability, String>, ParseError> {
        let mut result = HashMap::new();
        let xml = String::from_utf8_lossy(xml_bytes).to_string();
        let parsed = serde_llsd::from_str(&xml)?;

        if let Some(parsed_map) = parsed.as_map() {
            for (key, val) in parsed_map {
                let capability = Capability::from_string(key);
                if let LLSDValue::String(value) = val {
                    result.insert(capability, value.clone());
                }
            }
        }
        Ok(result)
    }
}

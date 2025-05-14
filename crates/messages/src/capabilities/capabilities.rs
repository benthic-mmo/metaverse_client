use core::fmt;
use std::fmt::Display;

use actix::Message;

#[derive(Debug, Clone)]
/// Describes the capabilities the client can have.
/// these are sent to the server to retrieve the capability endpoint.
pub enum Capability {
    /// Enable the viewer to get meshes.
    /// Used to retrieve the URL to get meshes from.
    GetMesh,
}
impl Display for Capability {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::GetMesh => write!(f, "GetMesh"),
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
    /// Constructs the XML of the capability request. I don't like this, but it works.
    pub fn new_capability_request(capabilities: Vec<Capability>) -> Self {
        let mut capability_string = "<llsd><array>".to_string();
        for capability in capabilities {
            capability_string.push_str("<string>");
            capability_string.push_str(&capability.to_string());
            capability_string.push_str("</string>");
        }
        capability_string.push_str("</array></llsd>");
        CapabilityRequest {
            capabilities: capability_string,
        }
    }
}

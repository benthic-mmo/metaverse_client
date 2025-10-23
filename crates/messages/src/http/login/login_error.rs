use serde::{Deserialize, Serialize};
use serde_llsd::LLSDValue;
use serde_llsd::converter::get;
use thiserror::Error;

use crate::errors::ParseError;

#[derive(Debug, Error)]
/// Conversion error for failed conversions
#[error("Conversion failed: {0}")]
pub struct ConversionError(pub &'static str);

#[derive(Debug, Clone, Serialize, Deserialize, Error)]
/// Contains the reason and message for a login failure. This represents an xml-rpc response that
/// is received in lieu of a LoginResponse object from the server after a failed login.
#[error("{message}")]
pub struct LoginError {
    /// Reason for the login failure
    pub reason: Reason,
    /// Message from the server about the login failure
    pub message: String,
}

impl From<ParseError> for LoginError {
    fn from(e: ParseError) -> Self {
        LoginError {
            reason: Reason::ParseError,
            message: e.to_string(),
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
/// Enum for login error reasons
pub enum Reason {
    /// User is already logged in
    Presence,
    /// Wrong username/password
    Key,
    /// Client lost connection
    Connection,
    /// Unknown
    Unknown,
    /// ParseError
    ParseError,
}
impl LoginError {
    /// convert from an LLSD map returned from XML parsing to a LoginError
    pub fn from_llsd(value: &LLSDValue) -> Self {
        match value {
            LLSDValue::Map(map) => {
                let reason: String = get("reason", map);
                let message: String = get("message", map);

                match reason.as_str() {
                    "presence" => LoginError {
                        reason: Reason::Presence,
                        message,
                    },
                    "key" => LoginError {
                        reason: Reason::Key,
                        message,
                    },
                    "connection" => LoginError {
                        reason: Reason::Connection,
                        message,
                    },
                    _ => LoginError {
                        reason: Reason::Unknown,
                        message,
                    },
                }
            }
            _ => LoginError {
                reason: Reason::Unknown,
                message: "Conversion failed".into(),
            },
        }
    }
}

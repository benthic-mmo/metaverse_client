use crate::str_val;
use std::error::Error;
/// the type for the conversionerror, thrown when failing to convert a login response to a struct
use std::fmt;

use serde::{Deserialize, Serialize};
use xmlrpc::Value;

#[derive(Debug)]
pub struct ConversionError(pub &'static str);

impl fmt::Display for ConversionError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Error for ConversionError {}

#[derive(Clone, Serialize, Deserialize)]
pub struct LoginError {
    pub reason: Reason,
    pub message: String,
}

impl LoginError {
    pub fn new(reason: Reason, message: &str) -> Self {
        LoginError {
            reason,
            message: message.to_string(),
        }
    }
}

impl fmt::Display for LoginError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let err_msg = match self.reason {
            Reason::Presence => {
                "Login failed because you are already logged in. Wait a few minutes and try again."
            }
            Reason::Key => "Username or password incorrect",
            Reason::Unknown => "Unknown error occured.",
            Reason::Connection => "Failed to connect to the server",
        };
        write!(f, "{}", err_msg)
    }
}
impl Error for LoginError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        None
    }
}
impl fmt::Debug for LoginError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "LoginError {{ reason: {}, message: {} }}",
            self.reason, self.message
        )
    }
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum Reason {
    Key,
    Presence,
    Unknown,
    Connection,
}
impl fmt::Display for Reason {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let msg = match self {
            Reason::Presence => "Presence",
            Reason::Key => "Key",
            Reason::Unknown => "Unknown",
            Reason::Connection => "Connection",
        };
        write!(f, "{}", msg)
    }
}

pub fn create_login_error_from_message(message: Value) -> LoginError {
    let xml_reason = str_val!(message["reason"]);
    let reason = match xml_reason {
        Some(reason) => match reason.as_str() {
            "presence" => Reason::Presence,
            "key" => Reason::Key,
            _ => Reason::Unknown,
        },
        None => Reason::Unknown,
    };
    let content = str_val!(message["message"]).expect("Unknown Message");

    LoginError::new(reason, &content)
}

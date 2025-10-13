use std::error::Error;
/// the type for the conversionerror, thrown when failing to convert a login response to a struct
use std::fmt;

use serde::{Deserialize, Serialize};
use serde_llsd::LLSDValue;

use crate::http::login::login_response::{get, FromLLSDValue};

#[derive(Debug)]
/// Conversion error for failed conversions
pub struct ConversionError(pub &'static str);

impl fmt::Display for ConversionError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Error for ConversionError {}

#[derive(Clone, Serialize, Deserialize)]
/// Login Error. Contains the reason and the message for failure
pub struct LoginError {
    /// The reason why the login failed
    pub reason: Reason,
    /// The message of the login failure
    pub message: String,
}

impl FromLLSDValue for LoginError {
    fn from_llsd(value: &LLSDValue) -> Option<Self> {
        match value {
            LLSDValue::Map(map) => {
                let reason: String = get("reason", map);
                let message: String = get("message", map);

                Some(match reason.as_str() {
                    "presence" => LoginError::new(Reason::Presence, &message),
                    "key" => LoginError::new(Reason::Key, &message),
                    "connection" => LoginError::new(Reason::Connection, &message),
                    _ => LoginError::new(Reason::Unknown, &message),
                })
            }
            _ => Some(LoginError::new(Reason::Unknown, "asdf")),
        }
    }
}

impl LoginError {
    /// create a new login error, using reason and message
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
                "Login failed because you are already logged in. Wait a few minutes and try again"
            }
            Reason::Key => "Username or password incorrect",
            Reason::Unknown => "Unknown error occured",
            Reason::Connection => "Connection error",
        };
        write!(f, "{} : {}", err_msg, self.message)
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
/// Reasons why the login failed
pub enum Reason {
    /// Login credentials were invalid
    Key,
    /// User is already logged in
    Presence,
    /// unknown
    Unknown,
    /// lost connection while logging in
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

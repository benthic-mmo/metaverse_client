use crate::{
    errors::errors::{AckError, CapabilityError, CircuitCodeError, CompleteAgentMovementError},
    login::login_errors::LoginError,
};
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// This represents errors that can arise from the mailbox failing to connect.
/// The mailbox is part of the client that handles packet IO and other logic to pass to the UI.
#[derive(Debug, Clone, Error, Serialize, Deserialize)]
#[error("{message}")]
pub struct MailboxSessionError {
    /// String message that contains error information
    pub message: String,
}
impl MailboxSessionError {
    /// Function for creating a new MailboxError
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

/// Represents errors that arise from failures within the session
#[derive(Clone, Debug, Error, Serialize, Deserialize)]
pub enum SessionError {
    /// this is sent when the CircuitCode that establishes the login fails.
    #[error("CircuitCodeError: {0}")]
    CircuitCode(#[from] CircuitCodeError),
    /// This is sent when the CompleteAgentMovement event fails
    #[error("CompleteAgentMovementError: {0}")]
    CompleteAgentMovement(#[from] CompleteAgentMovementError),
    /// This is sent when the Login event fails
    #[error("LoginError: {0}")]
    Login(#[from] LoginError),
    /// This is for when the mailbox fails to establish a session
    #[error("ClientConnectionError: {0}")]
    MailboxSession(#[from] MailboxSessionError),
    /// This is sent when Acknowledgement packets fail
    #[error("AckError: {0}")]
    AckError(#[from] AckError),
    /// This is sent when setting the capabilities fail
    #[error("CapabilityError: {0}")]
    Capability(#[from] CapabilityError),
}

impl SessionError {
    /// Create a new LoginError from the message's login error
    pub fn new_login_error(login_error: LoginError) -> Self {
        SessionError::Login(login_error)
    }

    /// to_bytes function for sending error from server to UI
    pub fn to_bytes(&self) -> Vec<u8> {
        bincode::serialize(self).expect("Failed to serialize SessionError")
    }
    /// from_bytes for sending error from server to UI
    pub fn from_bytes(bytes: &[u8]) -> Option<Self> {
        bincode::deserialize(bytes).ok()
    }
}

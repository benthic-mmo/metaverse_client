use crate::{http::login::login_error::LoginError, packet::message::UIMessage};
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
/// This represents the errors that can arise from CircuitCodes failing.
/// The CircuitCode is what the server returns after a successful login.
/// <https://wiki.secondlife.com/wiki/UseCircuitCode>
#[derive(Debug, Clone, Error, Serialize, Deserialize)]
#[error("{message}")]
pub struct CircuitCodeError {
    /// String message that contains error information
    pub message: String,
}

/// This represents errors that can arise from a CompleteAgentMovment event failing.
/// The CompleteAgentMovement packet is sent to finalize login.
/// <https://wiki.secondlife.com/wiki/CompleteAgentMovement>
#[derive(Debug, Clone, Error, Serialize, Deserialize)]
#[error("{message}")]
pub struct CompleteAgentMovementError {
    /// String message that contains error information
    pub message: String,
}

/// This represents errors that can arise from Acks failing.
/// Acks are sent to and from the server to verify packages got to their destination.
/// <https://wiki.secondlife.com/wiki/PacketAck>
#[derive(Debug, Clone, Error, Serialize, Deserialize)]
#[error("{message}")]
pub struct AckError {
    /// String message that contains error information
    pub message: String,
}

///Thrown when capabilities fail to get set
#[derive(Debug, Clone, Error, Serialize, Deserialize)]
#[error("{message}")]
pub struct CapabilityError {
    /// String message that contains error information
    pub message: String,
}

/// this is for returning SessionErrors that result from optoinal features
#[derive(Clone, Debug, Error, Serialize, Deserialize)]
pub enum FeatureError {
    #[error("Inventory: {0}")]
    /// for handling inventory related errors
    Inventory(String),
}

/// Represents errors that arise from failures within the core.
/// These are sent as packets to the UI to inform the UI of errors within the core.
#[derive(Clone, Debug, Error, Serialize, Deserialize)]
pub enum SessionError {
    /// this is sent when the CircuitCode that establishes the login fails.
    #[error("CircuitCodeError: {0}")]
    CircuitCode(#[from] CircuitCodeError),
    /// This is sent when the CompleteAgentMovement event fails
    #[error("CompleteAgentMovementError: {0}")]
    CompleteAgentMovement(#[from] CompleteAgentMovementError),
    /// This is sent when the Login event fails
    #[error("{0}")]
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

    /// This is sent when optional features fail
    #[error("FeatureError: {0}")]
    FeatureError(#[from] FeatureError),

    /// This is sent when files fail to create and IO errors are thrown
    #[error("IOError: {0}")]
    IOError(String),
}

// needed because serde can't serialize and deserialize the default std::io::error
impl From<std::io::Error> for SessionError {
    fn from(err: std::io::Error) -> Self {
        Self::IOError(err.to_string())
    }
}

impl UIMessage {
    /// Generates a new UIMessage to send to the UI from a SessionError
    pub fn new_session_error(data: SessionError) -> Self {
        UIMessage::Error(data)
    }
}

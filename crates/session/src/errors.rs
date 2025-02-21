use metaverse_messages::login_system::errors::LoginError;
use thiserror::Error;

/// This represents the errors that can arise from CircuitCodes failing.
/// The CircuitCode is what the server returns after a successful login.
/// https://wiki.secondlife.com/wiki/UseCircuitCode
#[derive(Debug, Clone, Error)]
#[error("{message}")]
pub struct CircuitCodeError {
    /// String message that contains error information
    pub message: String,
}
impl CircuitCodeError {
    /// Function for creating a new CircuitCodeError
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

/// This represents errors that can arise from a CompleteAgentMovment event failing.
/// The CompleteAgentMovement packet is sent to finalize login.
/// https://wiki.secondlife.com/wiki/CompleteAgentMovement
#[derive(Debug, Clone, Error)]
#[error("{message}")]
pub struct CompleteAgentMovementError {
    /// String message that contains error information
    pub message: String,
}
impl CompleteAgentMovementError {
    /// Function for creating a new CompleteAgentMovementError
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

/// This represents errors that can arise from Acks failing.
/// Acks are sent to and from the server to verify packages got to their destination.
/// https://wiki.secondlife.com/wiki/PacketAck
#[derive(Debug, Clone, Error)]
#[error("{message}")]
pub struct AckError {
    /// String message that contains error information
    pub message: String,
}
impl AckError {
    /// Function for creating a new AckError
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

/// This represents errors that can arise from the mailbox failing to connect.
/// The mailbox is part of the client that handles packet IO and other logic to pass to the UI.
#[derive(Debug, Clone, Error)]
#[error("{message}")]
pub struct MailboxError {
    /// String message that contains error information
    pub message: String,
}
impl MailboxError {
    /// Function for creating a new MailboxError
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

/// Represents errors that arise from failures within the session
#[derive(Clone, Debug, Error)]
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
    #[error("MailboxError: {0}")]
    Mailbox(#[from] MailboxError),
    /// This is sent when Acknowledgement packets fail
    #[error("AckError: {0}")]
    AckError(#[from] AckError),
}
impl SessionError {
    /// Create a new LoginError from the message's login error
    pub fn new_login_error(login_error: LoginError) -> Self {
        SessionError::Login(login_error)
    }
}

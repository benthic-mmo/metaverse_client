use serde::{Deserialize, Serialize};
use thiserror::Error;

/// This represents the errors that can arise from CircuitCodes failing.
/// The CircuitCode is what the server returns after a successful login.
/// <https://wiki.secondlife.com/wiki/UseCircuitCode>
#[derive(Debug, Clone, Error, Serialize, Deserialize)]
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
/// <https://wiki.secondlife.com/wiki/CompleteAgentMovement>
#[derive(Debug, Clone, Error, Serialize, Deserialize)]
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
/// <https://wiki.secondlife.com/wiki/PacketAck>
#[derive(Debug, Clone, Error, Serialize, Deserialize)]
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

///Thrown when capabilities fail to get set
#[derive(Debug, Clone, Error, Serialize, Deserialize)]
#[error("{message}")]
pub struct CapabilityError {
    /// String message that contains error information
    pub message: String,
}
impl CapabilityError {
    /// Function for creating a new CapabilityError
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

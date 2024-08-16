use metaverse_login::models::errors::LoginError;
use std::fmt;

#[derive(Clone)]
pub enum SessionError {
    CircuitCode(CircuitCodeError),
    CompleteAgentMovement(CompleteAgentMovementError),
    Login(LoginError),
    // Add other error types here
}

impl SessionError {
    pub fn new_login_error(login_error: LoginError) -> Self {
        SessionError::Login(login_error)
    }
    pub fn new(error: impl Into<SessionError>) -> Self {
        error.into()
    }
}

impl fmt::Display for SessionError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            SessionError::CircuitCode(err) => write!(f, "CircuitCodeError: {}", err),
            SessionError::Login(err) => write!(f, "LoginError: {}", err),
            SessionError::CompleteAgentMovement(err) => write!(f, "CompleteAgentMovement: {}", err),
            // Handle other error types here
        }
    }
}

#[derive(Clone)]
pub enum SendFailReason {
    Timeout,
    Unknown,
}
impl fmt::Display for SendFailReason {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let msg = match self {
            SendFailReason::Timeout => "Timeout",
            SendFailReason::Unknown => "Unknown",
        };
        write!(f, "{}", msg)
    }
}

#[derive(Clone)]
pub struct CircuitCodeError {
    pub reason: SendFailReason,
    pub message: String,
}
impl fmt::Display for CircuitCodeError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.reason)
    }
}

impl CircuitCodeError {
    pub fn new(reason: SendFailReason, message: String) -> Self {
        Self { reason, message }
    }
}

#[derive(Clone)]
pub struct CompleteAgentMovementError {
    pub reason: SendFailReason,
    pub message: String,
}

impl CompleteAgentMovementError {
    pub fn new(reason: SendFailReason, message: String) -> Self {
        Self { reason, message }
    }
}
impl fmt::Display for CompleteAgentMovementError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.reason)
    }
}

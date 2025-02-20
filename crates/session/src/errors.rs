use metaverse_messages::login::errors::LoginError;
use std::{error::Error, fmt};

#[derive(Clone, Debug)]
pub enum SessionError {
    CircuitCode(CircuitCodeError),
    CompleteAgentMovement(CompleteAgentMovementError),
    Login(LoginError),
    AckError(AckError), // Add other error types here
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
            SessionError::AckError(err) => write!(f, "AckError: {}", err),
        }
    }
}

impl Error for SessionError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            SessionError::CircuitCode(err) => Some(err),
            SessionError::Login(err) => Some(err),
            SessionError::CompleteAgentMovement(err) => Some(err),
            SessionError::AckError(err) => Some(err),
        }
    }
}

impl SessionError {
    pub fn as_boxed_error(&self) -> Box<dyn Error + Send + Sync> {
        match self {
            SessionError::CircuitCode(err) => Box::new(err.clone()) as Box<dyn Error + Send + Sync>,
            SessionError::Login(err) => Box::new(err.clone()) as Box<dyn Error + Send + Sync>,
            SessionError::CompleteAgentMovement(err) => {
                Box::new(err.clone()) as Box<dyn Error + Send + Sync>
            }
            SessionError::AckError(err) => Box::new(err.clone()) as Box<dyn Error + Send + Sync>,
        }
    }
}

#[derive(Clone, Debug)]
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

#[derive(Clone, Debug)]
pub struct CircuitCodeError {
    pub reason: SendFailReason,
    pub message: String,
}
impl fmt::Display for CircuitCodeError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.reason)
    }
}
impl Error for CircuitCodeError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        None
    }
}

impl CircuitCodeError {
    pub fn new(reason: SendFailReason, message: String) -> Self {
        Self { reason, message }
    }
}

#[derive(Clone, Debug)]
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
impl Error for CompleteAgentMovementError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        None
    }
}

#[derive(Clone, Debug)]
pub struct AckError {
    pub message: String,
}
impl AckError {
    pub fn new(message: String) -> Self {
        Self { message }
    }
}

impl fmt::Display for AckError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl Error for AckError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        None
    }
}

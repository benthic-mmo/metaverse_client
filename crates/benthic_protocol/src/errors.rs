use std::{io::Error, path::PathBuf};

use thiserror::Error;

use crate::messages::ui::{
    errors::{
        CapabilityError, CircuitCodeError, CompleteAgentMovementError, FeatureError,
        MailboxSessionError,
    },
    login_error::LoginError,
};

#[derive(Error, Debug)]
pub enum SessionError {
    #[error("Failed to create directory {dir}: {error}")]
    DirCreation { dir: PathBuf, error: Error },

    #[error("Failed to find benthic data directory")]
    NotFound {},

    #[error("Failed to write JSON: {error}")]
    JsonWriteError { error: serde_json::Error },

    #[error("IO Error: {0}")]
    IOError(#[from] std::io::Error),

    #[error("ClientConnectionError: {0}")]
    MailboxSession(#[from] MailboxSessionError),

    #[error("Login failed: {0}")]
    Login(#[from] LoginError),

    #[error("Feature failed: {0}")]
    Feature(#[from] FeatureError),

    #[error("Circuit code error: {0}")]
    CircuitCode(#[from] CircuitCodeError),

    #[error("Complete agent movement error: {0}")]
    CompleteAgentMovement(#[from] CompleteAgentMovementError),

    #[error("Capability error: {0}")]
    Capability(#[from] CapabilityError),
}

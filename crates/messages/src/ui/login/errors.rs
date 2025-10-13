use std::error::Error;
use thiserror::Error;

use crate::ui::login::login_errors;

#[derive(Debug, Error)]
/// Error types for handling LoginResponses
pub enum LoginResponseError {
    #[error("Failed to convert XMLRPC {0}")]
    /// Generic error for XMLRPC errors
    XLMRPCError(#[from] Box<dyn Error + Send + Sync>),

    #[error("failed to deserialize XMLRPC: {0}")]
    /// Generic error for std::io parsing
    DeserializeError(#[from] std::io::Error),

    #[error("Failed to convert {0}")]
    /// convert old login errors to new login errors
    /// TODO: remove this
    ConversionError(#[from] login_errors::ConversionError),
}

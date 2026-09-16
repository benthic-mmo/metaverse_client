use std::path::PathBuf;

use metaverse_messages::errors::ParseError;

#[derive(Debug, thiserror::Error)]
pub enum CapabilityError {
    #[error("Failed to send with SendRequestError: {0}")]
    SendRequestError(#[from] awc::error::SendRequestError),

    #[error("Failed to retrieve body: {0}")]
    PayloadError(#[from] awc::error::PayloadError),

    #[error("Capabilities failed to parse: {0}")]
    ParseError(#[from] ParseError),
}

#[derive(Debug, thiserror::Error)]
pub enum FilterAnimationError {
    #[error("failed to read animation file {path}: {source}")]
    ReadFile {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("failed to deserialize animation {path}: {source}")]
    Deserialize {
        path: PathBuf,
        #[source]
        source: serde_json::Error,
    },

    #[error("failed to create animation output {path}: {source}")]
    CreateOutput {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("failed to serialize animation {path}: {source}")]
    Serialize {
        path: PathBuf,
        #[source]
        source: serde_json::Error,
    },
}

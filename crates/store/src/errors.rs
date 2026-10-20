use std::time::SystemTimeError;

use awc::error::{PayloadError, SendRequestError};
use metaverse_messages::errors::ParseError;
use sqlx::migrate::MigrateError;
use thiserror::Error;
use uuid::Uuid;

/// Errors that can arise when retrieving outfit objects from the inventory
#[derive(Error, Debug)]
pub enum OutfitError {
    /// Failure to get current outfit version
    #[error("Failed to get current outfit version: {error}")]
    OutfitVersion {
        /// Wrapped inventory error
        error: InventoryError,
    },

    /// Failure to get current avatar version
    #[error("Failed to get current avatar version: {error}")]
    AvatarVersion {
        /// Wrapped inventory error
        error: InventoryError,
    },

    /// Failure to cache a new avatar
    #[error("Failed to insert new avatar to cache: {error}")]
    AvatarCacheFailure {
        /// Wrapped inventory error
        error: InventoryError,
    },

    /// Failure to retieve avatar data
    #[error("Failed to retrieve avatar data from cache: {error}")]
    AvatarDataRetrieveFailure {
        /// Wrapped inventory error
        error: InventoryError,
    },

    /// Failure to set outfit size for an agent
    #[error("Failed to set outfit size for {agent_id}: {error}")]
    OutfitSize {
        /// Agent ID of the failed outfit size setting
        agent_id: Uuid,
        /// Inventory error that caused the outfit size failure
        error: InventoryError,
    },

    /// Failure to retrieve inventory
    #[error("Failed to retrieve inventory {error}")]
    InventoryRetrieve {
        /// Wrapped InventoryError
        error: InventoryError,
    },
}

/// Errors that can arise fom sqlite inventory handling issues
#[derive(Error, Debug)]
pub enum InventoryError {
    /// Folder or Item LLSD failed to parse
    #[error("LLSD Error: {0}")]
    LLSDError(#[from] serde_llsd_benthic::errors::ParseError),

    /// Folder or Item LLSD failed to parse
    #[error("Parse Error: {0}")]
    ParseError(#[from] uuid::Error),

    /// Migration failed to run
    #[error("Migration Error: {0}")]
    MigrationError(#[from] MigrateError),

    /// Generic sqlite error
    #[error("Sqlx Error: {0}")]
    SqlxError(#[from] sqlx::Error),

    /// Folder or Item XML failed to parse
    #[error("XML error: {0}")]
    Xml(#[from] quick_xml::Error),

    /// Folder or Item UTF8 failed to parse
    #[error("UTF8 error: {0}")]
    Utf8(#[from] std::string::FromUtf8Error),

    /// Generic io error
    #[error("IO error : {0}")]
    IO(#[from] std::io::Error),

    /// Generic anyhow error. Likely from the benthic-serde-llsd crate
    #[error("Anyhow error: {0}")]
    Anyhow(#[from] anyhow::Error),

    /// String decode utf-8 error
    #[error("UTF-8 decode error: {0}")]
    StrUtf8(#[from] std::str::Utf8Error),

    /// Generic inventory error
    #[error("Inventory Error: {0}")]
    Error(String),

    /// Error from the messages crate
    #[error("Messges error: {0}")]
    Messages(#[from] ParseError),

    /// HTTP request error
    #[error("RequestError: {0}")]
    RequestError(#[from] SendRequestError),

    /// Error with the http payload
    #[error("PayloadError: {0}")]
    PayloadError(#[from] PayloadError),

    /// Generic serde error
    #[error("SerdeError: {0}")]
    SerdeError(#[from] serde_json::Error),

    /// Generic system time error
    #[error("System time error {0}")]
    SystemTimeError(#[from] SystemTimeError),

    /// Object ws not found in cache
    #[error("Cache Miss Error: {0}")]
    CacheMiss(String),
}

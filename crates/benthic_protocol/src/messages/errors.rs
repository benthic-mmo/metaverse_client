use serde_llsd_benthic::LLSDValue;
use std::{array::TryFromSliceError, str::Utf8Error, string::FromUtf8Error};
use thiserror::Error;

use crate::messages::ui::login_error::{LoginError, Reason};

#[derive(Debug, Error)]
/// Error handling for parsing throughout the messages crate
pub enum ParseError {
    #[error("Parse Error: {0}")]
    /// An error with a generic message
    Message(String),

    #[error("Missing field: {0}")]
    /// An error for when fields are not present in the parsed data
    MissingField(String),

    #[error("Invalid field:: {0}")]
    /// An error for when fields are invalid in the parsed data
    InvalidField(String),

    #[error("Failed to generate mesh: {0}")]
    /// An error thrown when meshes fail to parse
    MeshError(String),

    #[error("Failed to deserialize Serde-LLSD map")]
    /// An error thrown when serde-llsd fails to deserialize
    LLSDError(),

    #[error("Serde deserialze failed")]
    /// wrapper for serde errors
    SerdeError(#[from] serde_json::Error),

    #[error("Failed to decode slice")]
    /// wrapper for TryFromSlice errors
    SliceError(#[from] TryFromSliceError),

    #[error("Parse Error: {0}")]
    /// wrapper for std::io errors
    IOError(#[from] std::io::Error),

    #[error("Utf8Error: {0}")]
    /// wrapper for utf8 errors
    Utf8Error(#[from] Utf8Error),

    #[error("Anyhow Error: {0}")]
    /// wrapper for anyhow errors (used by serde-llsd)
    Anyhow(#[from] anyhow::Error),

    #[error("UUID parse error: {0}")]
    /// wrapper for uuid errors
    UuidError(#[from] uuid::Error),

    #[error("ParseInt error: {0}")]
    /// wrapper for parse int errors
    ParseInt(#[from] std::num::ParseIntError),

    #[error("Failed to parse bool: {0}")]
    /// wrapper for parse bool errors
    ParseBool(#[from] std::str::ParseBoolError),

    #[error("Failed to parse float: {0}")]
    /// wrapper forparse float errors
    ParseFloat(#[from] std::num::ParseFloatError),

    #[error("UTF8 error: {0}")]
    /// wrapper for FromUTF8 errors
    UTF8Error(#[from] FromUtf8Error),
}

impl From<ParseError> for LoginError {
    fn from(e: ParseError) -> Self {
        LoginError {
            reason: Reason::ParseError,
            message: e.to_string(),
        }
    }
}

impl From<LLSDValue> for ParseError {
    fn from(value: LLSDValue) -> Self {
        ParseError::InvalidField(format!("{value:?}"))
    }
}

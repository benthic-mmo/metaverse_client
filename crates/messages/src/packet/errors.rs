use std::{array::TryFromSliceError, str::Utf8Error, string::FromUtf8Error};

use bincode::error::DecodeError;
use thiserror::Error;

use crate::packet::header::PacketFrequency;

#[derive(Debug, Error)]
/// Errors that can arise when handling packets
pub enum PacketError {
    #[error("Unknown Packet ID: {id}, frequency: {frequency}")]
    /// Error type for handling unknown packets
    InvalidData {
        /// Packet ID of the invalid date
        id: u16,
        /// Frequency of the invalid data
        frequency: PacketFrequency,
    },

    #[error("StartPingCheck failed: {0}")]
    /// Errors that can arise from StartPingCheck
    StartPingCheckError(#[from] TryFromSliceError),

    #[error("UUID parse error: {0}")]
    /// UUID parsing error type
    UUIDParseError(#[from] uuid::Error),

    #[error("Parse Error:")]
    /// std::io error for handling IO parsing errors
    ParseError(#[from] std::io::Error),

    #[error("UTF8 parse error: {0}")]
    /// UTF8 parsing error type
    FromUTF8Error(#[from] FromUtf8Error),

    #[error("UTF8 parse error: {0}")]
    /// Slightly different UTF8 parsing error
    FromUTF8ErrorString(#[from] Utf8Error),

    #[error("Serde deserialize error: {0}")]
    /// Serde deserialize error
    SerdeDeserializeError(#[from] serde_json::Error),

    #[error("Serde deserialize error: {0}")]
    /// Slightly different serde deserialize error
    SerdeDeError(#[from] serde::de::value::Error),

    #[error("bincode deserialize error: {0}")]
    BincodeError(#[from] DecodeError),
}

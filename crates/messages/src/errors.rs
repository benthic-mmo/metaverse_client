use crate::packet::header::PacketFrequency;
use benthic_protocol::messages::errors::ParseError as ProtocolParseError;
use serde_llsd_benthic::LLSDValue;
use std::string::FromUtf8Error;
use thiserror::Error;

#[derive(Debug, Clone, Copy)]
pub struct ParseLocation {
    pub file: &'static str,
    pub line: u32,
    pub column: u32,
}

impl ParseLocation {
    #[track_caller]
    pub fn caller() -> Self {
        let location = std::panic::Location::caller();

        Self {
            file: location.file(),
            line: location.line(),
            column: location.column(),
        }
    }
}

impl std::fmt::Display for ParseLocation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}:{}", self.file, self.line, self.column)
    }
}

#[macro_export]
macro_rules! parse {
    ($expr:expr) => {{
        $expr.map_err(|e| $crate::errors::ParseError::Parse {
            location: $crate::errors::parse_error_location(),
            source: anyhow::Error::from(e),
        })
    }};
}

/// Return the caller's source location when debug parse errors are enabled.
#[track_caller]
pub fn parse_error_location() -> Option<ParseLocation> {
    #[cfg(feature = "debug-parse-errors")]
    {
        Some(ParseLocation::caller())
    }

    #[cfg(not(feature = "debug-parse-errors"))]
    {
        None
    }
}
fn format_parse_source(source: &anyhow::Error) -> String {
    source.to_string()
}
/// Format an optional source location.
pub fn format_parse_error_location(location: Option<ParseLocation>) -> String {
    match location {
        Some(location) => format!(" at {location}"),
        None => String::new(),
    }
}

/// Error handling for parsing throughout the messages crate.
#[derive(Debug, Error)]
pub enum ParseError {
    #[error(
    "Parse error{location}: {message}",
    location = format_parse_error_location(*location),
    message = format_parse_source(source)
    )]
    Parse {
        location: Option<ParseLocation>,

        #[source]
        source: anyhow::Error,
    },

    #[error("Unknown Packet ID: {id}, frequency: {frequency}")]
    UnknownPacket { id: u16, frequency: PacketFrequency },

    #[error("{0}")]
    SceneObjectParse(#[from] SceneObjectParseError),

    #[error(
        "Parse error{location}: {message}",
        location = format_parse_error_location(*location)
    )]
    Message {
        message: String,
        location: Option<ParseLocation>,
    },

    #[error(
        "Protocol parse error{location}",
        location = format_parse_error_location(*location)
    )]
    ProtocolParseError { location: Option<ParseLocation> },

    #[error(
        "Missing field{location}: {field}",
        location = format_parse_error_location(*location)
    )]
    MissingField {
        field: String,
        location: Option<ParseLocation>,
    },

    #[error(
        "Invalid field{location}: {field}",
        location = format_parse_error_location(*location)
    )]
    InvalidField {
        field: String,
        location: Option<ParseLocation>,
    },

    #[error(
        "Failed to generate mesh{location}: {message}",
        location = format_parse_error_location(*location)
    )]
    MeshError {
        message: String,
        location: Option<ParseLocation>,
    },

    #[error(
        "Failed to deserialize Serde-LLSD map{location}",
        location = format_parse_error_location(*location)
    )]
    LLSDError { location: Option<ParseLocation> },

    #[error(
        "UTF8 error{location}: {source}",
        location = format_parse_error_location(*location)
    )]
    UTF8Error {
        source: FromUtf8Error,
        location: Option<ParseLocation>,
    },
}

impl ParseError {
    #[track_caller]
    pub fn message(message: impl Into<String>) -> Self {
        Self::Message {
            message: message.into(),
            location: parse_error_location(),
        }
    }

    #[track_caller]
    pub fn missing_field(field: impl Into<String>) -> Self {
        Self::MissingField {
            field: field.into(),
            location: parse_error_location(),
        }
    }

    #[track_caller]
    pub fn invalid_field(field: impl Into<String>) -> Self {
        Self::InvalidField {
            field: field.into(),
            location: parse_error_location(),
        }
    }

    #[track_caller]
    pub fn llsd_error() -> Self {
        Self::LLSDError {
            location: parse_error_location(),
        }
    }

    #[track_caller]
    pub fn mesh_error(message: impl Into<String>) -> Self {
        Self::MeshError {
            message: message.into(),
            location: parse_error_location(),
        }
    }
}

#[derive(Debug, Error)]
#[error(
    "SceneObject parse failed{location}: {operation} failed for value `{value}`: {message}",
    location = format_parse_error_location(*location)
)]
pub struct SceneObjectParseError {
    pub operation: &'static str,
    pub value: String,
    pub location: Option<ParseLocation>,
    pub message: String,
}

impl SceneObjectParseError {
    #[track_caller]
    pub fn new(
        operation: &'static str,
        value: impl Into<String>,
        message: impl Into<String>,
    ) -> Self {
        Self {
            operation,
            value: value.into(),
            location: parse_error_location(),
            message: message.into(),
        }
    }
}

impl From<LLSDValue> for ParseError {
    #[track_caller]
    fn from(_: LLSDValue) -> Self {
        Self::LLSDError {
            location: parse_error_location(),
        }
    }
}

impl From<ProtocolParseError> for ParseError {
    #[track_caller]
    fn from(_: ProtocolParseError) -> Self {
        Self::ProtocolParseError {
            location: parse_error_location(),
        }
    }
}

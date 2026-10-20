use crate::packet::header::PacketFrequency;
use benthic_protocol::errors::ProtocolParseError;
use serde_llsd_benthic::LLSDValue;
use std::string::FromUtf8Error;
use thiserror::Error;

/// This is an error type that stores the file, line and location of errors.
/// this allows for the messages crate to be debugged much more efficiently.
#[derive(Debug, Clone, Copy)]
pub struct ParseLocation {
    /// File where the error came from
    pub file: &'static str,
    /// Line where the error came from
    pub line: u32,
    /// Column where the error came from
    pub column: u32,
}

impl ParseLocation {
    /// captures the file, line and location of errors.
    /// Track caller ensures that the error contains the information from the caller and doesn't
    /// point to this definition in the errors file.
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

/// Macro that converts errors to ParseErrors, saving their source and location.
///
/// The expression must return a result. On success its value remains unchanged. On failure it
/// returns a [`ParseEerror::Parse`] with the parsing location and original error
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
    /// Error for when parsing fails.
    /// this includes the location of the parse failure and the source, allowing for easier
    /// debugging.
    Parse {
        /// Location in the codebase of the error
        location: Option<ParseLocation>,

        /// error source
        #[source]
        source: anyhow::Error,
    },

    /// Packet is not recognized by benthic, identified by ID and Frequency
    #[error("Unknown Packet ID: {id}, frequency: {frequency}")]
    UnknownPacket {
        /// Packet ID
        id: u16,
        /// Frequency of the packet
        frequency: PacketFrequency,
    },

    /// Failed to parse scene object
    #[error("{0}")]
    SceneObjectParse(#[from] SceneObjectParseError),

    /// Parse error containing the error's location
    #[error(
        "Parse error{location}: {message}",
        location = format_parse_error_location(*location)
    )]
    Message {
        /// Error message from the failure
        message: String,
        /// Line, file and column ocation of the error
        location: Option<ParseLocation>,
    },

    /// Protocol parse error containing the error's location
    #[error(
        "Protocol parse error{location}",
        location = format_parse_error_location(*location)
    )]
    ProtocolParseError {
        /// Lne, file and column of the error
        location: Option<ParseLocation>,
    },

    /// Missing field error containing the error's location
    #[error(
        "Missing field{location}: {field}",
        location = format_parse_error_location(*location)
    )]
    MissingField {
        /// The field that is missing
        field: String,
        /// Line, file, and column of the error
        location: Option<ParseLocation>,
    },

    /// Invalid field error containing the error's location
    #[error(
        "Invalid field{location}: {field}",
        location = format_parse_error_location(*location)
    )]
    InvalidField {
        /// Field that is invalid
        field: String,
        /// line, file and column of the error
        location: Option<ParseLocation>,
    },

    /// Mesh handling error
    #[error(
        "Failed to generate mesh{location}: {message}",
        location = format_parse_error_location(*location)
    )]
    MeshError {
        /// mesh handling error message
        message: String,
        /// line file and column of the error
        location: Option<ParseLocation>,
    },

    /// LLSD parsing error
    #[error(
        "Failed to deserialize Serde-LLSD map{location}",
        location = format_parse_error_location(*location)
    )]
    LLSDError {
        /// line, file and column of the error
        location: Option<ParseLocation>,
    },

    /// UTF8 parsing error
    #[error(
        "UTF8 error{location}: {source}",
        location = format_parse_error_location(*location)
    )]
    UTF8Error {
        /// Wrapped UTF8 parsing error
        source: FromUtf8Error,
        /// line, file and column of the error
        location: Option<ParseLocation>,
    },
}

impl ParseError {
    /// Creates a parse error with a message and the caller's source location
    /// Message can be any type that implements [`Into<String>`]
    #[track_caller]
    pub fn message(message: impl Into<String>) -> Self {
        Self::Message {
            message: message.into(),
            location: parse_error_location(),
        }
    }

    /// Creates a parse error with caller's source location indicating a field is missing
    #[track_caller]
    pub fn missing_field(field: impl Into<String>) -> Self {
        Self::MissingField {
            field: field.into(),
            location: parse_error_location(),
        }
    }

    /// Creates a parse error with caller's source location indicating a field is invalid
    #[track_caller]
    pub fn invalid_field(field: impl Into<String>) -> Self {
        Self::InvalidField {
            field: field.into(),
            location: parse_error_location(),
        }
    }

    /// Creates a parse error with caller's source location indicating there was an LLSD error
    #[track_caller]
    pub fn llsd_error() -> Self {
        Self::LLSDError {
            location: parse_error_location(),
        }
    }

    /// Creates a parse error with a caller's source location indicating there was an error with the
    /// mesh
    #[track_caller]
    pub fn mesh_error(message: impl Into<String>) -> Self {
        Self::MeshError {
            message: message.into(),
            location: parse_error_location(),
        }
    }
}

/// Error type for failures when parsing a scene object
#[derive(Debug, Error)]
#[error(
    "SceneObject parse failed{location}: {operation} failed for value `{value}`: {message}",
    location = format_parse_error_location(*location)
)]
pub struct SceneObjectParseError {
    /// the operation that failed
    pub operation: &'static str,
    /// the value that failed when parsing
    pub value: String,
    /// the line, file and columb of the failure
    pub location: Option<ParseLocation>,
    /// The faulure's message
    pub message: String,
}

impl SceneObjectParseError {
    /// create a new SceneObjectParseError with the operation, value and message
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

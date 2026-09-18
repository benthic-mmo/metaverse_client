use awc::error::{PayloadError, SendRequestError};
use benthic_protocol::errors::SessionError;
use bitreader::BitReaderError;
use metaverse_messages::errors::ParseError;

/// Errors that can occur while processing a layer update.
#[derive(Debug, thiserror::Error)]
pub enum LayerError {
    /// An error occurred while processing a patch.
    #[error("PatchError: {0}")]
    PatchError(#[from] PatchError),

    /// An error occurred while accessing or communicating with the session.
    #[error("SessionError: {0}")]
    SessionError(#[from] SessionError),
}

/// Errors that can occur while decoding a terrain patch.
#[derive(Debug, thiserror::Error)]
pub enum PatchError {
    /// Failed to read patch data from the bit stream.
    #[error("BitReader error: {0}")]
    BitReader(#[from] BitReaderError),

    /// Failed to construct the patch header.
    #[error("Failed to create header: {0}")]
    Header(String),
}

/// Errors that can occur while retrieving or parsing simulator time.
#[derive(Debug, thiserror::Error)]
pub enum SimTimeError {
    /// The simulator time capability was not provided by the session.
    #[error("Capability not present")]
    CapNotPresent {},

    /// Failed to send the HTTP request.
    #[error("awc send request error: {0}")]
    SendRequest(#[from] SendRequestError),

    /// Failed to read or decode the HTTP response payload.
    #[error("awc payload error: {0}")]
    PayloadError(#[from] PayloadError),

    /// Failed to parse the simulator time response.
    #[error("parse error: {0}")]
    ParseError(#[from] ParseError),
}

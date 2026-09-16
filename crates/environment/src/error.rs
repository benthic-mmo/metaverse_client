use awc::error::{PayloadError, SendRequestError};
use benthic_protocol::errors::SessionError;
use bitreader::BitReaderError;
use metaverse_messages::errors::ParseError;

#[derive(Debug, thiserror::Error)]
pub enum LayerError {
    #[error("PatchError: {0}")]
    PatchError(#[from] PatchError),

    #[error("SessionError: {0}")]
    SessionError(#[from] SessionError),
}

#[derive(Debug, thiserror::Error)]
pub enum PatchError {
    #[error("BitReader error: {0}")]
    BitReader(#[from] BitReaderError),

    #[error("Failed to create header: {0}")]
    Header(String),
}

#[derive(Debug, thiserror::Error)]
pub enum SimTimeError {
    #[error("Capability not present")]
    CapNotPresent {},
    #[error("awc send request error: {0}")]
    SendRequest(#[from] SendRequestError),
    #[error("awc payload error: {0}")]
    PayloadError(#[from] PayloadError),

    #[error("parse error: {0}")]
    ParseError(#[from] ParseError),
}

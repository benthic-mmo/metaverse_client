use metaverse_messages::errors::ParseError;

#[derive(Debug, thiserror::Error)]
/// Errors for handling capability failures
pub enum CapabilityError {
    /// Capability send request failed
    #[error("Failed to send with SendRequestError: {0}")]
    SendRequestError(#[from] awc::error::SendRequestError),

    /// Failed to retrieve body of capability request
    #[error("Failed to retrieve body: {0}")]
    PayloadError(#[from] awc::error::PayloadError),

    /// Failed to parse capability error
    #[error("Capabilities failed to parse: {0}")]
    ParseError(#[from] ParseError),
}

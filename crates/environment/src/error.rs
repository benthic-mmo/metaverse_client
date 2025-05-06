use bitreader::BitReaderError;

/// The PatchError function. Stores errors thrown by the Patch handler
#[derive(Debug)]
pub struct PatchError {
    /// The message of the error
    pub message: String,
}

impl From<BitReaderError> for PatchError {
    fn from(err: BitReaderError) -> Self {
        PatchError {
            message: format!("BitReaderError: {:?}", err),
        }
    }
}

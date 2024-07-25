use std::error::Error;
/// the type for the conversionerror, thrown when failing to convert a login response to a struct
use std::fmt;

#[derive(Debug)]
pub struct ConversionError(pub &'static str);

impl fmt::Display for ConversionError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Error for ConversionError {}

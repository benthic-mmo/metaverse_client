use std::fmt;

pub enum ServerError {
    ConfigurationError(String),
    StartupError(String),
    IOError(String),
}
impl fmt::Display for ServerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "it's an error :(")
    }
}

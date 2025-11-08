use awc::error::{PayloadError, SendRequestError};
use metaverse_messages::errors::ParseError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum InventoryError {
    #[error("Sqlite error: {0}")]
    Sqlite(#[from] rusqlite::Error),

    #[error("XML error: {0}")]
    Xml(#[from] quick_xml::Error),

    #[error("UTF8 error: {0}")]
    Utf8(#[from] std::string::FromUtf8Error),

    #[error("IO error : {0}")]
    IO(#[from] std::io::Error),

    #[error("Anyhow error: {0}")]
    Anyhow(#[from] anyhow::Error),

    #[error("UTF-8 decode error: {0}")]
    StrUtf8(#[from] std::str::Utf8Error),

    #[error("Inventory Error: {0}")]
    Error(String),

    #[error("Messges error: {0}")]
    Messages(#[from] ParseError),

    #[error("RequestError: {0}")]
    RequestError(#[from] SendRequestError),

    #[error("PayloadError: {0}")]
    PayloadError(#[from] PayloadError),
}

use std::path::PathBuf;

use benthic_protocol::errors::SessionError;
use metaverse_cache::errors::InventoryError;
use metaverse_mesh::errors::MetaverseMeshError;
use metaverse_messages::{errors::ParseError, http::capabilities::Capability};

#[derive(Debug, thiserror::Error)]
pub enum MeshBuildError {
    #[error("Mesh Error: {0}")]
    MeshError(#[from] MetaverseMeshError),

    #[error("Inventory error: {0}")]
    InventoryError(#[from] InventoryError),
}


#[derive(Debug, thiserror::Error)]
pub enum DownloadError {
    #[error("Capbility missing: {capability}")]
    CapabilityNotPresent { capability: Capability },

    #[error("Response body empty")]
    EmptyBody {},

    #[error("Object contained no SceneGroup: {error}")]
    SceneGroupNotPresent { error: std::io::Error },

    #[error("Session error: {0}")]
    SessionError(#[from] SessionError),

    #[error("Inventory error: {0}")]
    InventoryError(#[from] InventoryError),

    #[error("IO Error: {0}")]
    IOError(#[from] std::io::Error),

    #[error("Parse Error: {0}")]
    ParseError(#[from] ParseError),

    #[error("Unwknown Pixel Format")]
    UnknownPixelFormatError {},
}

#[derive(Debug, thiserror::Error)]
pub enum ObjectUpdateError {
    #[error("Inventory error: {0}")]
    InventoryError(#[from] InventoryError),

    #[error("Session error: {0}")]
    SessionError(#[from] SessionError),

    #[error("Unknown Object Type: {object_type}")]
    UnknownObjectError { object_type: String },

    #[error("{feature} is not yet implemented")]
    Unimplemented { feature: String },
}

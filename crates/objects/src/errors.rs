use std::boxed;

use awc::error::PayloadError;
use benthic_protocol::errors::SessionError;
use metaverse_mesh::errors::MetaverseMeshError;
use metaverse_messages::{errors::ParseError, http::capabilities::Capability};
use metaverse_store::errors::InventoryError;

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

    #[error("Unknown Pixel Format")]
    UnknownPixelFormatError {},

    #[error("Jpeg2k error:")]
    Jpeg2kError(#[from] jpeg2k::error::Error),

    #[error("ImageError:")]
    ImageError(#[from] image::ImageError),

    #[error("PayloadError:")]
    PayloadError(#[from] PayloadError),

    #[error("Retryable Error: {error}")]
    Retryable {
        #[source]
        error: Box<DownloadError>,
    },

    #[error("Unrecoverable Download Error: {error}")]
    Unrecoverable {
        #[source]
        error: awc::error::SendRequestError,
    },

    #[error("Object download failed: {error}")]
    ObjectError {
        #[source]
        error: awc::error::SendRequestError,
    },

    #[error("Texture download failed: {error}")]
    TextureError {
        #[source]
        error: Box<DownloadError>,
    },
}
impl From<awc::error::SendRequestError> for DownloadError {
    fn from(error: awc::error::SendRequestError) -> Self {
        match &error {
            awc::error::SendRequestError::Connect(_) => DownloadError::Retryable {
                error: Box::new(DownloadError::ObjectError { error }),
            },

            _ => DownloadError::Unrecoverable { error },
        }
    }
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

    #[error("Serde error: {0}")]
    SerdeError(#[from] serde_json::Error),

    #[error("IO error: {0}")]
    IOError(#[from] std::io::Error),

    #[error("Mesh error: {0}")]
    MeshError(#[from] Box<dyn std::error::Error>),

    #[error("Metaverse Mesh error: {0}")]
    MetaverseMeshError(#[from] metaverse_mesh::errors::MetaverseMeshError),
}

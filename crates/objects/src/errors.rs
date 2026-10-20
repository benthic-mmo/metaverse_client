use awc::error::PayloadError;
use benthic_protocol::errors::ProtocolSessionError;
use metaverse_mesh::errors::MetaverseMeshError;
use metaverse_messages::{errors::ParseError, http::capabilities::Capability};
use metaverse_store::errors::InventoryError;

use crate::errors;

/// Errors that can occur when building meshes for metaverse assets
#[derive(Debug, thiserror::Error)]
pub enum MeshBuildError {
    /// Mesh could not be generated
    #[error("Mesh Error: {0}")]
    MeshError(#[from] MetaverseMeshError),

    /// An inventory operation failed
    #[error("Inventory error: {0}")]
    InventoryError(#[from] InventoryError),

    /// A session operation failed
    #[error("Session Error: {0}")]
    SessionError(#[from] ProtocolSessionError),

    /// A mesh failed to parse
    #[error("Failed to parse mesh: {0}")]
    ParseError(#[from] metaverse_messages::errors::ParseError),

    /// An object failed to download
    #[error("Failed to download: {0}")]
    DownloadError(#[from] DownloadError),
}

/// Errors that can occur when downloading assets from the endpoint  
#[derive(Debug, thiserror::Error)]
pub enum DownloadError {
    /// Capability isn't present to download from
    #[error("Capbility missing: {capability}")]
    CapabilityNotPresent {
        /// Capability that is not present
        capability: Capability,
    },

    /// Response body was empty
    #[error("Empty response body: status={status}, url={url}, headers={headers:?}")]
    EmptyBody {
        /// status code of the response
        status: u16,
        /// URL of the response
        url: String,
        /// Headers associated with the empty response
        headers: Vec<(String, String)>,
    },

    /// Asset was not found at url
    #[error("Asset not found: 404 at url={url}")]
    NotFound {
        /// URL of the missing asset
        url: String,
    },

    /// Other HTTP status faiure
    #[error("Asset request failed: HTTP {status}, url={url}")]
    HttpStatus {
        /// Status code of the failed http request
        status: u16,
        /// URL of the failed http request
        url: String,
    },

    /// Object contained no SceneGroup
    #[error("Object contained no SceneGroup: {error}")]
    SceneGroupNotPresent {
        /// IO error of the failed scenegroup
        error: std::io::Error,
    },

    /// A session operation failed
    #[error("Session error: {0}")]
    SessionError(#[from] ProtocolSessionError),

    /// An inventory operation failed
    #[error("Inventory error: {0}")]
    InventoryError(#[from] InventoryError),

    /// An IO operation failed
    #[error("IO Error: {0}")]
    IOError(#[from] std::io::Error),

    /// An asset failed to parse
    #[error("Parse Error: {0}")]
    ParseError(#[from] ParseError),

    /// An image failed to parse beacuse of an unknown pixel
    #[error("Unknown Pixel Format")]
    UnknownPixelFormatError {},

    /// A JPEG2000 failure
    #[error("Jpeg2k error:")]
    Jpeg2kError(#[from] jpeg2k::error::Error),

    /// An image failed to parse
    #[error("ImageError:")]
    ImageError(#[from] image::ImageError),

    /// An error with the payload
    #[error("PayloadError:")]
    PayloadError(#[from] PayloadError),

    /// Objects that fail to download but should be retried get handled as a Retryable download error.
    #[error("Retryable Error: {error}")]
    Retryable {
        #[source]
        /// The error that can be retried
        error: Box<DownloadError>,
    },

    /// Objects that fail to download permanently (like a 404) get handled as unrecoverable downlod
    /// errors
    #[error("Unrecoverable Download Error: {error}")]
    Unrecoverable {
        #[source]
        /// the error that cannot be retried
        error: awc::error::SendRequestError,
    },

    /// Send request error
    #[error("Object download failed: {error}")]
    ObjectError {
        #[source]
        /// AWC error that failed
        error: awc::error::SendRequestError,
    },

    /// Textures that fail to download get handled as Retryable or Unrecoverable.
    /// This identifies which texture failures cannot be re-attempted.
    #[error("Texture download failed: {error}")]
    TextureError {
        #[source]
        /// Texture download error for the failure
        error: Box<DownloadError>,
    },

    /// Error for handling unimplemented features
    /// One day this will be gone...
    #[error("Feature not implemented: {feature}")]
    Unimplemented {
        /// String describing the unimplemented feature
        feature: String,
    },

    /// Errors coming from the metaverse mesh crate
    #[error("Metaverse Mesh Error: {0}")]
    MetaverseMeshError(#[from] MetaverseMeshError),
}
/// This converts SendRequestErrors to REtryable or Unrecoverable errors.
/// If it is a connect error, or a timeout error, it is retryable. re-queue, and try again.
/// If it is any other kind of error, it is unrecoverable.
impl From<awc::error::SendRequestError> for DownloadError {
    fn from(error: awc::error::SendRequestError) -> Self {
        match &error {
            awc::error::SendRequestError::Connect(_) => DownloadError::Retryable {
                error: Box::new(DownloadError::ObjectError { error }),
            },
            awc::error::SendRequestError::Timeout => DownloadError::Retryable {
                error: Box::new(DownloadError::ObjectError { error }),
            },

            _ => DownloadError::Unrecoverable { error },
        }
    }
}

/// Errors that can arise from object updates
#[derive(Debug, thiserror::Error)]
pub enum ObjectUpdateError {
    /// An inventory operation failed
    #[error("Inventory error: {0}")]
    InventoryError(#[from] InventoryError),

    /// A session operation failed
    #[error("Session error: {0}")]
    SessionError(#[from] ProtocolSessionError),

    /// An object of unknown type was parsed
    #[error("Unknown Object Type: {object_type}")]
    UnknownObjectError {
        /// the type of the object that failed to parse
        object_type: String,
    },

    /// Error for handling unmplimented features
    #[error("{feature} is not yet implemented")]
    Unimplemented {
        /// a string describing the unimplemented feature
        feature: String,
    },

    /// Object failed to parse using Serde
    #[error("Serde error: {0}")]
    SerdeError(#[from] serde_json::Error),

    /// An IO error happened
    #[error("IO error: {0}")]
    IOError(#[from] std::io::Error),

    /// There was an error when handing a mesh
    #[error("Mesh error: {0}")]
    MeshError(#[from] Box<dyn std::error::Error>),

    /// An error from the metaverse mesh crate
    #[error("Metaverse Mesh error: {0}")]
    MetaverseMeshError(#[from] metaverse_mesh::errors::MetaverseMeshError),

    /// A download failed
    #[error("Download Error: {0}")]
    DownloadError(#[from] errors::DownloadError),

    /// An image faile to handle
    #[error("ImageError: {0}")]
    ImageError(#[from] image::ImageError),

    /// A mesh is too small to create geometry
    #[error("Mesh is too small to create geometry")]
    MeshTooSmall,
}

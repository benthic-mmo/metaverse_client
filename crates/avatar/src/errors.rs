use std::path::PathBuf;

use benthic_protocol::errors::SessionError;
use metaverse_mesh::errors::MetaverseMeshError;
use uuid::Uuid;

#[derive(Debug, thiserror::Error)]
pub enum AvatarError {
    #[error("{feature}: currently unimplemented")]
    Unimplemented { feature: String },

    #[error("Inventory is not currently initialized")]
    InventoryUninitialized {},

    #[error("Agent not found for agent_id: {agent}")]
    AgentNotFound { agent: Uuid },

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Serde Error: {0}")]
    SerdeError(#[from] serde_json::Error),

    #[error("Session Error: {0}")]
    SessionError(#[from] SessionError),

    #[error("Mesh Error: {0}")]
    MetaverseMeshError(#[from] MetaverseMeshError),

    #[error("Avatar mesh generation failed: {error}")]
    MeshGeneration { error: String },

    #[error("Avatar {agent_id} not yet fully loaded")]
    NotLoaded { agent_id: Uuid },

    #[error("Avatar {agent_id} not yet in scene")]
    NotPresent { agent_id: Uuid },
}

#[derive(Debug, thiserror::Error)]
pub enum AnimationError {
    #[error("{feature} is currently unimplmented")]
    Unimplemented { feature: String },

    #[error("Session Error: {0}")]
    SessionError(#[from] SessionError),

    #[error("Failed to read file: {path}, {source}")]
    ReadFile {
        path: PathBuf,
        source: std::io::Error,
    },

    #[error("failed to deserialize animation {path}: {source}")]
    Deserialize {
        path: PathBuf,
        #[source]
        source: serde_json::Error,
    },

    #[error("failed to create animation output {path}: {source}")]
    CreateOutput {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("failed to serialize animation {path}: {source}")]
    Serialize {
        path: PathBuf,
        #[source]
        source: serde_json::Error,
    },

    #[error("Mesh Error: {0}")]
    MeshError(#[from] MetaverseMeshError),
}

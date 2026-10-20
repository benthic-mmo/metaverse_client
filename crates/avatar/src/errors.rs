use benthic_protocol::errors::ProtocolSessionError;
use metaverse_mesh::errors::{MetaverseMeshAnimationError, MetaverseMeshError};
use uuid::Uuid;

/// Errors that can arise from handling avatars
#[derive(Debug, thiserror::Error)]
pub enum AvatarError {
    /// Error for features that are unimplemented
    #[error("{feature}: currently unimplemented")]
    Unimplemented {
        /// Short string describing what feature is uniplemented
        feature: String,
    },

    /// Inventory is not initialized
    /// TODO: This should really be current outfit is not initialized
    #[error("Inventory is not currently initialized")]
    InventoryUninitialized {},

    /// Agent is not found in the session
    #[error("Agent not found for agent_id: {agent}")]
    AgentNotFound {
        /// missing agent's uuid
        agent: Uuid,
    },

    /// Wrapper for IO errors
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    /// Wrapper for serde errors
    #[error("Serde Error: {0}")]
    SerdeError(#[from] serde_json::Error),

    /// wrapper for session errors
    #[error("Session Error: {0}")]
    SessionError(#[from] ProtocolSessionError),

    /// Wrapper for metaverse mesh errors
    #[error("Mesh Error: {0}")]
    MetaverseMeshError(#[from] MetaverseMeshError),

    /// Avatar mesh failed to create
    #[error("Avatar mesh generation failed: {error}")]
    MeshGeneration {
        /// String text of the mesh generation error
        error: String,
    },

    /// Avatar is not currently loaded in
    #[error("Avatar {agent_id} not yet fully loaded")]
    NotLoaded {
        /// ID of the agent not loaded
        agent_id: Uuid,
    },

    /// Avatar is not present in the scene yet
    #[error("Avatar {agent_id} not yet in scene")]
    NotPresent {
        /// ID of the agent not present
        agent_id: Uuid,
    },

    /// Avatar contains an unknown joint
    #[error("Unknown Joint in skeleton: {joint}")]
    UnknownJoint {
        /// String name of the joint that is missing
        joint: String,
    },
}

/// Enum describing errors that can arise when handling animations
#[derive(Debug, thiserror::Error)]
pub enum AnimationError {
    /// Wrapper for session errors
    #[error("Session Error: {0}")]
    SessionError(#[from] ProtocolSessionError),

    /// Error for features that are unimplemented
    #[error("{feature} is currently unimplmented")]
    Unimplemented {
        /// Short string describing the unimplemented feature
        feature: String,
    },

    /// Wrapper for animation errors coming from the metaverse mesh crate
    #[error("Metaverse Mesh Animation Error: {0}")]
    AnimationError(#[from] MetaverseMeshAnimationError),

    /// Wrapper for mesh errors coming from the metavarse mesh crate
    #[error("Mesh Error: {0}")]
    MeshError(#[from] MetaverseMeshError),
}

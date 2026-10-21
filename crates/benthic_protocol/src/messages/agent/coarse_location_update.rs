use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
/// Defines locations of agent dots on the minimap.
pub struct MinimapEntities {
    /// the x location of the agent on the minimap
    x: u8,
    /// the y location of the agent on the minimap
    y: u8,
    /// the z location of the agent on the minmap
    z: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
/// Struct to contain the ID of you, and the agent you are following
pub struct CoarseLocationUpdate {
    /// The xyz locations of agents
    pub locations: Vec<MinimapEntities>,
    /// the ID of the user
    pub you: i16,
    /// the ID of the user you are following
    pub prey: i16,
}

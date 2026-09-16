use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
/// Defines locations of agent dots on the minimap.
pub struct MinimapEntities {
    x: u8,
    y: u8,
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

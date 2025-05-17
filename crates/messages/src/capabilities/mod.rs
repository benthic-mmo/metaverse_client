//! # Capabilities
//! <https://wiki.secondlife.com/wiki/Capabilities>
//!
//! OpenSimulator uses HTTP endpoints for retrieving large amounts of data. These are called
//! Capabilities or "caps" by the spec. These endpoints are unique URLs that can send and receive
//! data.
///
///
/// Informs the server ofw which capabilities are required, and retrieves the endpoint URLs
pub mod capabilities;

/// Handles the folder structure for the inventory capability response.
pub mod folder_types;

/// Handles the types for inventory items
pub mod item_types;

pub mod item_data;

pub mod scene_object;

pub mod mesh_data;

//! # Metaverse Store
//!
//! This crate contains functions and handlers for maintaining a local sqlite store for handling
//! avatar, object and inventory cache for open metaverse projects.
//!
//!# Goals
//!- Create an easy to use cache system for open metaverse projects
//!
//! ## Current Status
//! <div class="warning">Work In Progress</div>
//! This crate is under active development, and is not suitable for production use. APIs may change
//! frequently, and many protocol features are currently unimplemented.
#![warn(missing_docs)]
use metaverse_messages::utils::object_types::ObjectType;
use std::path::PathBuf;
use uuid::Uuid;

/// Outfit item object used by avatars to manage appearance
/// TODO: this should not be here, why is it in lib
#[derive(Debug)]
pub struct OutfitItem {
    /// Name of the outfit object
    pub name: String,
    /// ID of the outfit object
    pub item_id: Uuid,
    /// asset ID of the ouft object (UUID of the mesh)
    pub asset_id: Uuid,
    /// Type of the object
    pub item_type: ObjectType,
    /// Path to the object's cached json on disk
    pub json_path: PathBuf,
    /// path to the object's cached mesh on disk
    pub mesh_path: PathBuf,
}
/// Functions for dealing with the object and avatar cache
pub mod cache;
/// Errors for the store crate
pub mod errors;
/// Functions for initializing sqlite and running migrations
pub mod initialize_sqlite;
/// functions for initializing and handling inventory
pub mod inventory;

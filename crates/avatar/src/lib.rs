//! # Metaverse Avatars
//!
//! This crate contains functionality for handling metaverse avatars. Handling their skeleton,
//! building animiations, and adding objects to outfits worn by them.
//!
//! ## Goals
//! - Represent avatar objects in an easy to use way, and build straightforward interfaces for
//! managing them
//!
//!## Current Status
//! <div class="warning">Work In Progress</div>
//! This crate is under active development, and is not suitable for production use. APIs may change
//! frequently, and many protocol features are currently unimplemented.
#![warn(missing_docs)]
/// structs and enum definitions for metaverse avatars
pub mod avatar;

/// functions for creating and updating the avatar global skeleton
pub mod skeleton;

/// Error handling for avatars
pub mod errors;

/// functions for handling object updates targeting avatars
pub mod avatar_object_handler;

/// Functions for handling animation retargeting for avatars
pub mod animation;

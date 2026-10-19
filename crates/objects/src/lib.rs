//! # Metaverse Object
//!
//! This crate contains functionality for handling objects created by open metaverse projects. this
//! includes handling parametric primitives, sculpt textures, and meshes.
//!
//! # Goals
//! - Create an easy to use abstraction layer between the complicated opensim protocol and a simpler
//!   benthic protocol
//! ## Current Status
//! <div class="warning">Work In Progress</div>
//! This crate is under active development, and is not suitable for production use. APIs may change
//! frequently, and many protocol features are currently unimplemented.
#![warn(missing_docs)]
/// Contains functions for downloading assets from capability endpoints.
pub mod download_asset;
/// error handling for the Objects crate
pub mod errors;
/// Functions for handling mesh data
pub mod mesh;
/// Functions for handling object updates
pub mod object_updates;
/// Functions for generating parametric prim meshes from object update data
pub mod parametric_prim;
/// Functions for generating sculpt object meshes from object update data
pub mod sculpt_object;
/// utility functions
pub mod utils;

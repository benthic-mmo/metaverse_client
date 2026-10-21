//! # Metaverse Protocol
//!
//! This crate contains common structs and functions used by both the benthic protocol and the
//! messages crate. This crate's function is to create a new, simplified protocol for handling open
//! metaverse applications through the benthic core.
//!
//! ## Goals
//! - Define a simplified protocol for open metaverse applications to build frontend projects with
//!   using Benthic as the core.
//! - Unify server-side legacy structs with simplified benthic protocol structs
//!
//! ## Current Status
//! <div class="warning">Work In Progress</div>
//! This crate is under active development, and is not suitable for production use. APIs may change
//! frequently, and many protocol features are currently unimplemented.
#![warn(missing_docs)]
/// Common data for handling animations
pub mod animation;
/// Common data for handling avatars
pub mod avatar;
/// Error handling for the protocol crate
pub mod errors;
/// UI messages and responses
pub mod messages;
/// Common data for handling objects
pub mod objects;
/// Session data
pub mod session;

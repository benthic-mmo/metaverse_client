//! # Metaverse Core
//!
//! This crate contains a modular, asyncronous backend for open metaverse viewers. The core's scope
//! is to handle all packet IO between the core and the server, and between the core and the UI,
//! allowing UI projects to only have to worry about display and managing player input.
//!
//! ## Goals
//! - Enable completely async client-server processing using the actix actor system
//! - Fully seperate UI and core concerns, using inter-process UDP packets to allow communication
//! between the server and the player
//! - Fully seperate core and packet parsing conerns, using only well-defined structs throughout the
//! project
//! - Handle as much mesh and vertex math as possible in the core, allowing UI and mesh generation
//! to be handled with as few gotchas as possible
//! - Complete and understandable documentation
//! - Create developer-friendly APIs that can be used in lieu of the more confusing
//! opensimulator/secondlife APIs.
//!
//!## Current Status
//! <div class="warning">Work In Progress</div>
//! This crate is under active development, and is not suitable for production use. APIs may change
//! frequently, and many protocol features are currently unimplemented.
#![warn(missing_docs)]
/// Handles mailbox events to do with handling avatars
pub mod avatar;
/// Handles mailbox events required for establishing viewer capabilities
pub mod capabilities;
/// Handles mailbox events for generating land and environment
pub mod environment;
/// This module stores custom error definitions
pub mod errors;
/// This module initializes the mailbox
pub mod initialize;
/// Handles mailbox events for handling and updating inventory
pub mod inventory;
/// Handles mailbox events for retrieving and rendering objects
pub mod objects;
/// Handles mailbox events required for opening and maintaining the session
pub mod session;
/// handles packet sending between UI and core, and core and server
pub mod transport;

//! Async library for open metaverse clients!
//!
//! Provides an easy, asyncronous way of interacting with open metaverse servers in Rust.
//! Handles packet IO between client and server, exposing a UDS interface for simple integration
//! into other projects.
//! This is meant to operate as an easily-bootstrappable backend for new open metaverse viewer projects.
//!
//! <div class="warning">This crate is in development!</div>
//! This isn't ready for any kind of serious use yet! Check back later for updates!

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

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
/// This module is for running in your client to subscribe to the server events
pub mod client_subscriber;
/// This module contains the errors for the project
pub mod errors;
/// This module initializes the mailbox
pub mod initialize;
/// This module handles packet IO and logic
pub mod mailbox;
/// This module is to allowe the server to receive messages from the UI. 
pub mod server_subscriber;

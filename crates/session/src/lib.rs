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
/// This module handles packet IO and logic
/// serves as the core of the project, handling scheduling and packet processing
pub mod core;
/// This module is to allow the server to receive messages from the UI.
pub mod core_subscriber;
/// This module stores custom error definitions
pub mod errors;
/// This module handles sending http requests to the enabled capability endpoints.
pub mod http_handler;
/// This module initializes the mailbox
pub mod initialize;
/// This module handles all incoming UDP packets from the server
pub mod udp_handler;
/// This module is for running in your client to subscribe to the server events
pub mod ui_subscriber;

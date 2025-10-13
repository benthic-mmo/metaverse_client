//! # Metaverse Messages
//!
//!
//! A straightforward library for server to client and client to server IO for the open metaverse.
//!
//! Desgined to be modular enough that the same library can be used to write client applications
//! and server applications, allowing spec changes to be immediately reflected in viewers.
//!
//! <div class="warning"> This crate is in development! </div>
//! Currently does not have all of the packet types, and does not fully implement all server side
//! packet to_bytes functions. Contributions welcome!

#![warn(missing_docs)]
/// Errors
/// Contains errors that can arise when handling packets
pub mod errors;

/// Definitions for things related to the packet, such as the packet itself and headers.
pub mod packet;
/// Packets related to UI
/// Many of these are not packets in the open metaverse spec, and only exist for sending messages
/// from the core to the UI.
pub mod ui;
/// Utilities. Mostly constants used throughout the crate.
pub mod utils;

/// HTTP packet definitions
pub mod http;
pub mod legacy;
/// UDP packet definitions
pub mod udp;

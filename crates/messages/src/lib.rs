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
/// Packets related to user agents.
/// Contains packets for updating agent location, and handling appearance, movmement and outfits.
pub mod agent;
/// Packets related to chat.
/// Contains packets for sending and receiving chat messages.
pub mod chat;
/// Packets related to the core functionality of the client and server.
/// Conains packets related to sending and receiving acks, sending disconnects, and handling region
/// handshakes.
pub mod core;
/// Packets related to environment rendering
/// Contains packets related to Layer and object handling.
pub mod environment;
/// Errors
/// Contains errors that can arise when handling packets
pub mod errors;
/// Packets related to login.
/// this also contains functions used for parsing and handling the XMLRPC login functionality.
/// Though not strictly a packet, this will be used by both the client and server.
pub mod login;
/// Definitions for things related to the packet, such as the packet itself and headers.
pub mod packet;
/// Packets related to UI
/// Many of these are not packets in the open metaverse spec, and only exist for sending messages
/// from the core to the UI.
pub mod ui;
/// Utilities. Mostly constants used throughout the crate.
pub mod utils;

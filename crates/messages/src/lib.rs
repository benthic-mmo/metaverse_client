//! # Metaverse Messages
//!
//! This crate contains packet definitions for packets sent to and from open metaverse servers. The
//! messages crate's scope is to handle all parsing functionality for both incoming and outgoing
//! packets on a metaverse server, allowing the same crate to be used for viewer and server
//! applications.
//!
//! ## Goals
//! - Reflect as closely as possible the naming conventions and struct layout of existing open
//! metaverse packets
//! - Efficiently decode into rust structs using sparing external types  
//! - Maintain clear and complete documentation for all packets and fields
//! - Implement both encoding and decoding, allowing the crate to be used both in server and viewer
//! applications
//!
//! ## Current Status
//! <div class="warning">Work In Progress</div>
//! This crate is under active development, and is not suitable for production use. APIs may change
//! frequently, and many protocol features are currently unimplemented.
#![warn(missing_docs)]
/// Definitions for errors that can arise when handling packets
pub mod errors;
/// Definitions for things related to the packet, such as the packet itself and headers.
pub mod packet;
/// Packets related to UI
/// Many of these are not packets in the open metaverse spec, and only exist for sending messages
/// from the core to the UI.
pub mod ui;
/// Utilities. Mostly constants used throughout the crate.
pub mod utils;
// HTTP packet definitions
pub mod http;
/// Legacy packet definitions. These are no longer used.
pub mod legacy;
/// UDP packet definitions
pub mod udp;

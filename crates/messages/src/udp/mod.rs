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

/// Packets related to object handling
pub mod object;

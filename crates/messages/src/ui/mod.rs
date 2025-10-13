/// Used for informing the UI that the core has received a layer that should be rendered.
/// Contains the path to the generated gltf file, and the position to render it.
/// This allows all terrain generation to be handled by the client, allowing for very small packets
/// to be sent between the UI and the client.
pub mod mesh_update;

/// Errors created from messages and the session that should be from the core to the UI.
pub mod errors;

/// Packets related to login.
/// this also contains functions used for parsing and handling the XMLRPC login functionality.
/// Though not strictly a packet, this will be used by both the client and server.
pub mod login;

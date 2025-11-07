/// Used for informing the UI that the core has received a layer that should be rendered.
/// Contains the path to the generated gltf file, and the position to render it.
/// This allows all terrain generation to be handled by the client, allowing for very small packets
/// to be sent between the UI and the client.
pub mod mesh_update;

/// Errors created from messages and the session that should be from the core to the UI.
pub mod errors;

/// UI message for sending the UI relevant login response data
pub mod login_response;

/// UI message for sending a chat from the viewer to the client
pub mod chat_from_viewer;
/// UI message for requesting a login
pub mod login_event;
/// UI message for requesting a logout
pub mod logout;

pub mod land_update;

/// Used for managing packets that are meant to be sent directly to the UI.
/// Not all of these are real packets in the spec, though some of them are.
pub mod ui_events;

/// Used for informing the UI that the core has received a layer that should be rendered.
/// Contains the path to the generated gltf file, and the position to render it.
/// This allows all terrain generation to be handled by the client, allowing for very small packets
/// to be sent between the UI and the client.
pub mod layer_update;
/// Used for sending a login from the UI to the core. Contains the first name, last name,
/// password, viewer name, and grid.
pub mod login;

/// Errors created from messages and the session that should be from the core to the UI.
pub mod errors;

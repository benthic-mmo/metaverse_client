//! # Rust implementation of OpenSimulator's [login protocol](http://opensimulator.org/wiki/SimulatorLoginProtocol)
///
///

/// Errors that can be thrown upon login. Not part of any packet spec.
pub mod login_errors;

/// handles parsing the login response xml that is sent back from the server.
pub mod login_response;

/// Stores the simulator login protocol that is used for creating a login call to the server.
///implemented from the protocol as defined by <http://opensimulator.org/wiki/SimulatorLoginProtocol>
pub mod simulator_login_protocol;

/// error handling for login objects
pub mod errors;

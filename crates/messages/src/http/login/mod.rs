//! # Rust implementation of OpenSimulator's [login protocol](http://opensimulator.org/wiki/SimulatorLoginProtocol)

/// Error that can be thrown upon login as the alternative to receiving a valid login response
pub mod login_error;

/// handles parsing the login response xml that is sent back from the server.
pub mod login_response;

/// Stores the simulator login protocol that is used for creating a login call to the server.
///implemented from the protocol as defined by <http://opensimulator.org/wiki/SimulatorLoginProtocol>
pub mod simulator_login_protocol;

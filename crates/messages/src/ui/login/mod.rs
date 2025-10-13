//! # Rust implementation of OpenSimulator's [login protocol](http://opensimulator.org/wiki/SimulatorLoginProtocol)
///
///
/// # Logout Request
/// <https://wiki.secondlife.com/wiki/LogoutRequest>
///
/// Logs out user from the simulator and ends the session
///
/// ## Header
/// |CompleteAgentMovement |||||
/// |----------------------|---------|-----------------|------------------|-----------------|
/// | Packet Header        | id: 252 | reliable: true  | zerocoded: false | frequency: Low  |
///
/// ## Packet Structure
/// | LogoutRequest         |          |                    |                                 |
/// |-----------------------|----------|--------------------|---------------------------------|
/// | AgentID               | 16 bytes | [Uuid](uuid::Uuid) | The ID of the user agent        |  
/// | SessionID             | 16 bytes | [Uuid](uuid::Uuid) | The ID of the current session   |  
///
pub mod logout_request;

/// Errors that can be thrown upon login. Not part of any packet spec.
pub mod login_errors;

/// handles parsing the login response xml that is sent back from the server.
pub mod login_response;

/// Handles sending the login xml that initiates login
pub mod login_xmlrpc;

/// Stores the simulator login protocol that is used for creating a login call to the server.
///implemented from the protocol as defined by <http://opensimulator.org/wiki/SimulatorLoginProtocol>
pub mod simulator_login_protocol;

/// error handling for login objects
pub mod errors;

//! # Rust implementation of OpenSimulator's [login protocol](http://opensimulator.org/wiki/SimulatorLoginProtocol)
///
///
/// # Circuit Code
/// <https://wiki.secondlife.com/wiki/UseCircuitCode>
///
/// Sent from the viewer to establish a circuit connection with a simulator. This is necessary
/// before any other circuit commmunication is possible.
/// The simulator will start sending StartPingCheck messages after this is sent
///
/// ## Header
/// | UseCircuitCode |||||
/// |--------------|---------------|----------------|-------------------|---------------------|
/// | Packet Header| id:3          | reliable: false| zerocoded: false  | frequency: Low      |
///
/// ## Packet Structure
/// | CircuitCodeData |         |       |               |
/// |-----------------|---------|-------|---------------|
/// | Code            | 4 bytes | [u32] | The code the server will check against other trusted packets |
/// | SessionID       | 16 bytes| [Uuid](uuid::Uuid) | The ID of the session |
/// | ID              | 16 bytes| [Uuid](uuid::Uuid) | undocumented          |
pub mod circuit_code;

/// # Complete Agent Movement
/// <https://wiki.secondlife.com/wiki/CompleteAgentMovement>
///
/// This establishes the avatar presence in a region. If this packet is not sent, the avatar never
/// appears, and the login does not fully succeed.
///
/// ## Header
/// |CompleteAgentMovement |||||
/// |----------------------|---------|-----------------|------------------|-----------------|
/// | Packet Header        | id: 249 | reliable: false | zerocoded: false | frequency: Low  |
///
/// ## Packet Structure
/// | CompleteAgentMovement |          |                    |                                 |
/// |-----------------------|----------|--------------------|---------------------------------|
/// | AgentID               | 16 bytes | [Uuid](uuid::Uuid) | The ID of the user agent (sent from server to viwer with login)|
/// | SessionID             | 16 bytes | [Uuid](uuid::Uuid) | The ID of the user session (sent from server to client with login)|
/// | CircuitCode           | 4 bytes  | [u32]              | The CircuitCode (sent from server to client with login) |
pub mod complete_agent_movement;

/// Errors that can be thrown upon login. Not part of any packet spec.
pub mod login_errors;

/// handles parsing the login response xml that is sent back from the server.
pub mod login_response;

/// Handles sending the login xml that initiates login
pub mod login_xmlrpc;

pub mod logout_request;
/// Stores the simulator login protocol that is used for creating a login call to the server.
///implemented from the protocol as defined by <http://opensimulator.org/wiki/SimulatorLoginProtocol>
pub mod simulator_login_protocol;

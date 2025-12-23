/// # Teleport Request
/// <https://wiki.secondlife.com/wiki/TeleportRequest>
///
/// Initiates a teleport for the avatar to a new region or location.
///
/// ## Header
/// | CompleteAgentMovement |||||
/// |----------------------|--------|-----------------|------------------|-----------------|
/// | Packet Header        | id: 62 | reliable: true  | zerocoded: false | frequency: Low  |
///
/// ### Packet Structure
/// | Field      | Size     | Type               | Description                     |
/// |------------|----------|--------------------|---------------------------------|
/// | agent_id   | 16 bytes | [Uuid](uuid::Uuid) | the ID of the agent teleporting |
/// | session_id | 16 bytes | [Uuid](uuid::Uuid) | the ID of the session requesting teleport|
/// | region_id  | 16 bytes | [Uuid](uuid::Uuid) | The ID of the destination region |
/// | position_x | 4 bytes  | [f32]| the x position of the destination in the region |
/// | position_y | 4 bytes  | [f32]| the y position of the destination in the region |
/// | position_z | 4 bytes  | [f32]| the z position of the destination in the region |
/// | look_at_x  | 4 bytes  | [f32]| the x position of the look direction in the region|
/// | look_at_y  | 4 bytes  | [f32]| the y position of the look direction in the region|
/// | look_at_z  | 4 bytes  | [f32]| the z position of the look direction in the region|
pub mod teleport_request;

/// # Teleport Start
/// <https://wiki.secondlife.com/wiki/TeleportStart>
///
/// Sent by the simulator to indicate that the teleport process has begun.
///
/// ## Header
/// | CompleteAgentMovement |||||
/// |----------------------|--------|-----------------|------------------|-----------------|
/// | Packet Header        | id: 73 | reliable: true  | zerocoded: false | frequency: Low  |
///
/// ### Packet Structure
/// | Field          | Size    | Type                  | Description                             |
/// |----------------|---------|-----------------------|-----------------------------------------|
/// | teleport_flags  | 4 bytes | [u32] | Flags indicating teleport type and status |
pub mod teleport_start;

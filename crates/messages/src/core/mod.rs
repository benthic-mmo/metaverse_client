/// # Disable Simulator
/// <https://wiki.secondlife.com/wiki/DisableSimulator>
///
/// A packet sent by the server to inform the viewer of disconnection. This packet has no body, and
/// is functionally just a header.
///
/// ## Header
/// | DisableSimulator   |       |                |                   |                     |
/// |--------------|-------------|----------------|-------------------|---------------------|
/// | Packet Header| id:152      | reliable: false| zerocoded: false  |     frequency: Low  |
pub mod disable_simulator;

/// # Packet Ack
/// <https://wiki.secondlife.com/wiki/PacketAck>
///
/// The acknowledgement packet sent from the viewer to verify receiving reliable packets.
///
/// ## Header
/// | PacketAck    |             |                |                   |                     |
/// |--------------|-------------|----------------|-------------------|---------------------|
/// | Packet Header| id:251      | reliable: false| zerocoded: false  | frequency: Fixed    |
///
///
/// ## Packet Structure
/// | PacketAck ||||
/// |--------|--------|------|-----|
/// | count  | 1 byte  | [u8]  | number of ack IDs contained in the packet|
/// | Packet Ids| variable bytes | ID | List of IDs that need to be acked |  
/// | ID     | 4 bytes | [u32] | Sequence numbers of packets to be acked  |
pub mod packet_ack;

/// # Region Handshake
/// <https://wiki.secondlife.com/wiki/RegionHandshake>
///
/// The packet sent from the server in response to CompleteAgentMovement. The viewer responds to
/// this with RegionHandshakeReply, which finishes the handshakes and begins object updats with
/// CoarseLocationUpdate.
///
/// ## Header
/// | Region Handshake   |       |                |                   |                     |
/// |--------------|-------------|----------------|-------------------|---------------------|
/// | Packet Header| id:80       | reliable: false| zerocoded: false  |     frequency: Low  |
///
/// ## Packet Structure
/// | Region Handshake     |         |       |              |
/// |----------------------|---------|-------|--------------|
/// | RegionFlags          | 4 bytes | [u32] | Region flags |
/// | Unused byte          | 1 byte  |       | unknown, but must be read to keep alignment |
/// | SimAccess            | 1 byte  | [u8]  | The access level of the viewer |
/// | NameLength           | 1 byte  | [u8]  | the length in bytes of the sim name |
/// | SimName              | variable bytes| [String] | The name of the simulator |
/// | SimOwner             | 4 bytes | [Uuid](uuid::Uuid) | The user ID of the owner of the sim|
/// | IsEstateManager      | 4 bytes | [bool]| is the user an estate manager of this sim |
/// | WaterHeight          | 4 bytes | [f32] | the height of the water in the sim |
/// | Billablefactor       | 4 bytes | [f32] | undocumented |
/// | CacheID              | 16 bytes | [Uuid](uuid::Uuid) | undocumented |
/// | TerrainBase0         | 16 bytes | [Uuid](uuid::Uuid) | undocumented |
/// | TerrainBase1         | 16 bytes | [Uuid](uuid::Uuid) | undocumented |
/// | TerrainBase2         | 16 bytes | [Uuid](uuid::Uuid) | undocumented |
/// | TerrainBase3         | 16 btyes | [Uuid](uuid::Uuid) | undocumented |
/// | TerrainDetail0       | 16 bytes | [Uuid](uuid::Uuid) | undocumented |
/// | TerrainDetail1       | 16 bytes | [Uuid](uuid::Uuid) | undocumented |
/// | TerrainDetail2       | 16 bytes | [Uuid](uuid::Uuid) | undocumented |
/// | TerrainDetail3       | 16 bytes | [Uuid](uuid::Uuid) | undocumented |
/// | TerrainStartHeight0  | 4 bytes | [f32] | undocumented |
/// | TerrainStartheight1  | 4 bytes | [f32] | undocumented |
/// | TerrainStartheight2  | 4 bytes | [f32] | undocumented |
/// | TerrainStartheight3  | 4 bytes | [f32] | undocumented |
/// | TerrainHeightRange0  | 4 bytes | [f32] | undocumented |
/// | TerrainHeightRange1  | 4 bytes | [f32] | undocumented |
/// | TerrainHeightRange2  | 4 bytes | [f32] | undocumented |
/// | TerrainHeightRange3  | 4 bytes | [f32] | undocumented |
pub mod region_handshake;

/// # Region Handshake Reply
/// <https://wiki.secondlife.com/wiki/RegionHandshakeReply>
///
/// The viewer sends this in response to RegionHandshake, which begins object updates via
/// CoarseLocationUpdate. This finishes the login handshake.
///
/// ## Header
/// | Region Handshake Reply |   |                |                   |                     |
/// |--------------|-------------|----------------|-------------------|---------------------|
/// | Packet Header| id:149      | reliable: false| zerocoded: false  |     frequency: Low  |
///
/// ## Packet Structure
/// | Region Handshake |          |       |              |
/// |------------------|----------|-------|--------------|
/// | AgentID          | 16 bytes | [Uuid](uuid::Uuid) | user's agent ID |
/// | SessionID        | 16 bytes | [Uuid](uuid::Uuid) | User's session ID |
/// | Flags            | 4 bytes  | [u32] | undocumented |
pub mod region_handshake_reply;

/// # Start Ping Check
/// <https://wiki.secondlife.com/wiki/StartPingCheck>
///
/// Used to emasure ping times. PingId is increased by 1 each time StartPingCheck is sent. at 255
/// it rolls over to 0.
///
/// ## Header
/// | StartPingCheck  |          |                |                   |                     |
/// |--------------|-------------|----------------|-------------------|---------------------|
/// | Packet Header| id:1        | reliable: false| zerocoded: false  |     frequency: High |
///
/// ## Packet Structure
/// | PingCheck ||||
/// |--------|--------|------|-----|
/// | PingID | 1 byte | [u8] | the ID of the ping being sent. Used by complete ping check to verify it was received.|
/// | OldestUnacked | 4 bytes | [u32] | The sequence number of the most recent message sent by the source, stored as little-endian |
pub mod start_ping_check;

/// # Complete Ping Check
/// <https://wiki.secondlife.com/wiki/CompletePingCheck>
///
/// The response sent by the viewer when receiving a ping check packet from the server.
/// Completes the check started by the viewer.
///
/// ## Header
/// | CompletePingCheck  |       |                |                   |                     |
/// |--------------|-------------|----------------|-------------------|---------------------|
/// | Packet Header| id:2        | reliable: false| zerocoded: false  |     frequency: High |
///
///
/// ## Packet Structure
/// | PingCheck ||||
/// |--------|--------|------|-----|
/// | PingID | 1 byte | [u8] | the value received during StartPingCheck. Lets server know which ping was completed. |
pub mod complete_ping_check;

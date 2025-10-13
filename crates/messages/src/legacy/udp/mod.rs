/// # Agent Wearables Request
/// <https://wiki.secondlife.com/wiki/AgentWearablesRequest>
///
/// This packet is used to request the wearables of a user.
/// THIS IS A LEGACY PACKET!!! The correct way to handle wearables requests is done using the
/// FetchInventoryDescendents2 and FetchLibDescendents2 capability endpoints.
///
/// ## Header
/// | Agent Wearables Request|   |                |                   |                     |
/// |--------------|-------------|----------------|-------------------|---------------------|
/// | Packet Header| id:381      | reliable: false| zerocoded: false  | frequency: Low      |
///
/// ## Packet Structure
/// | Agent Wearables Request ||||
/// |--------------|----------|--------------------|-------------------|
/// | agent_id     | 16 bytes | [Uuid](uuid::Uuid) | ID of the user    |
/// | session_id   | 16 bytes | [Uuid](uuid::Uuid) | ID of the session |
pub mod agent_wearables_request;

/// # Agent Wearabales Update
/// <https://wiki.secondlife.com/wiki/AgentWearablesUpdate>
///
/// This packet is the response to the agent wearables request packet. This used to contain the
/// wearables of the requested user.
/// THIS IS A LEGACY PACKET!!! The correct way to handle wearables requests is done using the
/// FetchInventoryDescendents2 and FetchLibDescendents2 capability endpoints.
///
/// ## Header
/// | Agent Wearables Update|    |                |                   |                     |
/// |--------------|-------------|----------------|-------------------|---------------------|
/// | Packet Header| id:382      | reliable: false| zerocoded: false  | frequency: Low      |
///
/// ## Packet Structure
/// | Agent Wearables Update ||||
/// |--------------|----------|--------------------|-------------------|
/// | agent_id     | 16 bytes | [Uuid](uuid::Uuid) | ID of the user    |
/// | session_id   | 16 bytes | [Uuid](uuid::Uuid) | ID of the session |
/// | serial_num   | 4 bytes  | [u32]| Serial number of the packet. Used to prevent handling packets out of order. |
/// | wearable count| 1 byte  | [u8] |  Number of wearables contained in the packet |
/// | Wearables    | variable bytes || Wearables contained in the packet |
///
/// ### Wearable
/// This is the wearable data contained within the packet. It holds information about the wearable,
/// such as how to retrieve it from the server, what type it is, and what its ID is.
/// | Wearable ||||
/// |--------------|----------|--------------------|-------------------|
/// | item_id      | 16 bytes | [Uuid](uuid::Uuid) | ID of the item, used for inventory |
/// | asset_id     | 16 bytes | [Uuid](uuid::Uuid) | ID of the asset, used for retrieving from the ViewerAsset endpoint.|
/// | wearable_type| 1 byte   | [u8] | Type of the wearable, such as shape, or shirt, etc... |
pub mod agent_wearables_update;

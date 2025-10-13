/// # AgentUpdate
/// <https://wiki.secondlife.com/wiki/AgentUpdate>
///
/// The agent update packet. used for sending the server data about the player's curernt state.
/// Must be sent from the viewer to the server on a regular basis in order to know the agent's position and velocity.
/// Sent up to ten times per second
///
/// ## Header
/// | AgentUpdate  |             |                |                   |                     |
/// |--------------|-------------|----------------|-------------------|---------------------|
/// | Packet Header| id:5        | reliable: false| zerocoded: false  |     frequency: High |
///
/// ## Packet Structure
/// Each AgentUpdate packet sends an AgentData struct, containing information about what is being
/// updated.
/// | AgentData     |         |             |   |
/// |---------------|---------|-------------|---|
/// | AgentID       |16 bytes | [Uuid](uuid::Uuid) | ID of the user agent   |
/// | SessionID     |16 bytes | [Uuid](uuid::Uuid) | ID of the user session |
/// | BodyRotation  |16 bytes | [Quaternion](glam::Quat)  | where the user's body is facing|
/// | HeadRotation  |16 bytes | [Quaternion](glam::Quat) | Where the user's head is facing|
/// | State         |1 byte   | [u8]          | Typing or editing states |
/// | CameraCenter  |12 bytes | [Vector3](glam::Vec3)[[u8]] | Location of the camera in region local coordinates|
/// | CameraAtAxis  |12 bytes | [Vector3](glam::Vec3)[[u8]] | X rotational axis of the camera (forward) |
/// | CameraLeftAxis|12 bytes | [Vector3](glam::Vec3)[[u8]] | Y rotational axis of the camera (Left) |
/// | CameraUpAxis  |12 bytes | [Vector3](glam::Vec3)[[u8]] | Z rotational axis of the camera (Up) |
/// | Far           |4 bytes  | [f32]         | Distance the viewer can see in meters |
/// | ControlFlags  |4 bytes  | [u32]         | Events such as movement and standing up |
/// | Flags         |1 bytes  | [u8]          | Flag for hiding group tile in nametag |
pub mod agent_update;

/// # CoarseLocationUpdate
/// <https://wiki.secondlife.com/wiki/CoarseLocationUpdate>
///
/// This packet is used to populate the minimap with green indicators to represent where the agents
/// are. Z-azis is multiplied by 4 to obtain true Z location.
/// The "you" and "Prey" tell the client which ones in the index you are, and the person you are
/// following, if you are following someone.
///
/// ## Header
/// | CoarseLocationUpdate  |             |                |                   |                     |
/// |--------------|-------------|----------------|-------------------|---------------------|
/// | Packet Header| id:6        | reliable: false| zerocoded: false  | frequency: Medium   |
///
/// ## Packet Structure
/// Each coarse location update packet comes with a a struct containing an XYZ location of other
/// agents, and an index.
///
/// |CoarseLocationUpdateData|                  |             |                         |
/// |------------------------|------------------|-------------|-------------------------|
/// | Locations              | List of Locations|             |                         |
/// | Locations\[Location\]    | 12 bytes         | [Vector3](glam::Vec3)[[u8]] | XYZ location of an agent|
/// | You                    | 2 bytes          | [i16]         | Index of you in the list|
/// | Prey                   | 2 bytes          | [i16]         | Index of who you are following in the list|
pub mod coarse_location_update;

/// # Avatar Appearance
/// <https://wiki.secondlife.com/wiki/AvatarAppearance>
///
/// This packet is used to update the appearane of a user's avatar.
/// Contains textures and VisualParams for the avatar
///
/// ## Header
/// | Avatar Appearance  |       |                |                   |                     |
/// |--------------|-------------|----------------|-------------------|---------------------|
/// | Packet Header| id:158      | reliable: false| zerocoded: false  | frequency: Low      |
///
/// ## Packet Structure
/// |Avatar Appearance ||||
/// |--------------|----------|--------------------|----------------|
/// | id           | 16 bytes | [Uuid](uuid::Uuid) | ID of the user |  
/// | is_trial     | 1 byte   | [bool]             | Is the user a trial user|
/// | texture_len  | 2 bytes  | [u16]              | length of the texture data block |
/// | texture_data | variable bytes |              | Texture data for each face |
/// | v_param_len  | 1 byte   | [u8]               | length of visual param block |
/// | visual_param_data | variable byes |          | Bytes containing the visual param data |
pub mod avatar_appearance;

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

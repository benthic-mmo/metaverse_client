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
/// # Object Update
/// <https://wiki.secondlife.com/wiki/ObjectUpdate>
///
/// Sent by the server to update an object. Handles avatars, clothes, furniture, trees, grass, etc.
/// This notifies the viewer of almost everything. While not being a very large packet, nearly
/// every byte is used individually, and there aren't many multi-byte data structures to read.
///
/// ## Header
/// | ObjectUpdate |             |                |                  |                     |
/// |--------------|-------------|----------------|------------------|---------------------|
/// | Packet Header| id:12       | reliable: false| zerocoded: true  |     frequency: High |
///
/// ## Packet Structure
/// | ObjectUpdate ||||
/// |---------------|---------|-------|--------------------------------------|
/// | region_x      | 4 bytes | [u32] | Global x coordinate of the simulator |
/// | region_y      | 4 bytes | [u32] | Global y coordinate of the simulator |
/// | time_dilation | 2 bytes | [u16] | The current lag from the server. Used by physics simulations to keep up with real time. |
/// | id            | 4 bytes | [u32] | region local ID. used for most operations in lieu of the object's full UUID. |
/// | state         | 1 byte  | [u8]  | Unused except by grass to determine grass species |
/// | full_id       | 16 bytes| [Uuid](uuid::Uuid) | The full UUID of the object |
/// | crc           | 4 bytes | [u32] | CRC values. Not currently checked against anything. |
/// | pcode         | 1 byte  | [u8]  | Type of object represented by the packet. Avatar, grass, tree, etc |
/// | material      | 1 byte  | [u8]  | Type of material the object is made of. Wood, plastic, flesh, etc |
/// | click_action  | 1 byte  | [u8]  | The default action taken when the object is clicked. Open, sit, etc |
/// | scale_x       | 4 bytes | [u32] | The x value of the object's scale |
/// | scale_y       | 4 bytes | [u32] | The y value of the object's scale |
/// | scale_z       | 4 bytes | [u32] | the z value of the object's scale |
/// | data_length   | 1 byte  | [u8]  | Number of bytes to read for the object_data |
/// | [object_data](#object-data) | variable bytes | | Velocity, position and rotation values|
/// | parent_id     | 4 bytes | [u32] | Local ID of an object this object is a child of. 0 if none is present. |
/// | update_flags  | 4 bytes | [u32] | Gives various pieces of information to the viewer, like empty inventory or scripted |
/// | [Primitive geometry](#primitive-geometry) | 23 bytes | | Data for the viewer to draw primitive objects |
/// | texture_length| 1 byte  | [u8]  |  Number of bytes to read for the texture_entry data |
/// | texture_entry | variable bytes || Full property list of each object's face, including textures and colors. |
/// | anim_length   | 1 byte  | [u8]  | Number of bytes to read for the texture_anim_data |
/// | texture_anim  | variable bytes || Properties to set up texture animations of the face of the object |
/// | name_length   | 2 bytes | [u16] | Number of bytes to read for the name value. Big-endian |
/// | name_value    | variable bytes || Name value pairs specific to the object. Used for avatar names. |
/// | data_length   | 2 bytes | [u16] | Number of bytes to read for the generic appended data |
/// | data          | variable bytes || Generic appended data |
/// | text_length   | 1 byte  | [u8]  | Number of bytes to read for the text data |
/// | text          | variable bytes || Text that hovers over the object |
/// | text_color_r  | 1 byte  | [u8]  | Hover text color's red value |
/// | text_color_g  | 1 byte  | [u8]  | Hover text color's green value |
/// | text_color_b  | 1 byte  | [u8]  | Hover text color's blue value |
/// | text_color_a  | 1 byte  | [u8]  | Hover text color's alpha value |
/// | media_length  | 1 byte  | [u8]  | Number of bytes to read for the media URL |
/// | media_url     | variable bytes || URL for any media attached to the object. Will always be a webpage. |
/// | particle_len  | 1 byte  | [u8]  | Number of bytes to read for the particle system data |
/// | particle_system | variable bytes || Attached particles for the object |
/// | extra_len     | 1 byte  | [u8]  | Number of bytes to read for the extra parameters |
/// | extra_params  | variable bytes || Data related to flexible primitives, sculpt data or light |
/// |[sound](#sound)| 41 bytes | | Data for looping sound the object emits |
/// | joint_type    | 1 byte  | [u8]  | Type of joint the object uses. Legacy.|
/// | joint_pivot_x | 4 bytes | [f32] | x location of pivot. Legacy. |
/// | joint_pivot_y | 4 bytes | [f32] | y location of pivot. Legacy. |
/// | joint_pivot_z | 4 bytes | [f32] | z location of pivot. Legacy. |
/// | joint_axis_or_anchor_x | 4 bytes | [f32] | x location of the offset or axis. Legacy |
/// | joint_axis_or_anchor_y | 4 bytes | [f32] | y location of the offset or axis. Legacy |
/// | joint_axis_or_anchor_z | 4 bytes | [f32] | z location of the offset or axis. Legacy |
///
/// ## Motion Data
/// MotionData is read differently based on the length of the objectadata field.
/// There are high precision updates, medium precision updates and low precision updates,
/// which read different sized integers. High precision updates use the largest amount of
/// bytes, using f32s for its value, and low precision updates use the smallest amount of bytes,
/// using u8s for its value.
///
/// ### High Precision Update with Foot Collision Plane
///
/// | Motion Data       | 76 bytes|||
/// |-------------------|---------|-------|--------------------------------------|
/// | collision_plane_a | 4 bytes | [f32] | a corner of the collision plane      |
/// | collision_plane_b | 4 bytes | [f32] | b corner of the collision plane      |
/// | collision_plane_c | 4 bytes | [f32] | c corner of the collision plane      |
/// | collision_plane_d | 4 bytes | [f32] | d corner of the collision plane      |
/// | [High Precision Update](#high-precision-update) |||                        |
///
/// The remaining 60 bytes are used to create a high precision update. However, the last 3 f32s are not read.
/// the angular velocity when receiving a 76 byte object data will always be 0.0, 0.0, 0.0.
///
/// ### High Precision Update
/// | Motion Data       | 60 bytes|||
/// |-------------------|---------|-------|--------------------------------------|
/// | position_x        | 4 bytes | [f32] | the x position of the object         |
/// | position_y        | 4 bytes | [f32] | the y position of the object         |
/// | position_z        | 4 bytes | [f32] | the z position of the object         |
/// | velocity_x        | 4 bytes | [f32] | the x velocity of the object         |
/// | velocity_y        | 4 bytes | [f32] | the y velocity of the object         |
/// | velocity_z        | 4 bytes | [f32] | the z velocity of the object         |
/// | acceleration_x    | 4 bytes | [f32] | the x acceleration of the object     |
/// | acceleration_y    | 4 bytes | [f32] | the y acceleration of the object     |
/// | acceleration_z    | 4 bytes | [f32] | the z acceleration of the object     |
/// | rotation_x        | 4 bytes | [f32] | the x rotation of the object as a quaternion |
/// | rotation_y        | 4 bytes | [f32] | the y rotation of the object as a quaternion |
/// | rotation_z        | 4 bytes | [f32] | the z rotation of the object as a quaternion |
/// | rotation_w        | 4 bytes | [f32] | the w rotation of the object as a quaternion |
/// | angular_velocity_x| 4 bytes | [f32] | the x angular velocity of the object |
/// | angular_velocity_y| 4 bytes | [f32] | the y angular velocity of the object |
/// | angular_velocity_z| 4 bytes | [f32] | the z angular velocity of the object |
///
/// ### Medium Precision Update With Foot Collision Plane
/// | Motion Data       | 48 bytes|||
/// |-------------------|---------|-------|--------------------------------------|
/// | collision_plane_a | 4 bytes | [f32] | a corner of the collision plane      |
/// | collision_plane_b | 4 bytes | [f32] | b corner of the collision plane      |
/// | collision_plane_c | 4 bytes | [f32] | c corner of the collision plane      |
/// | collision_plane_d | 4 bytes | [f32] | d corner of the collision plane      |
/// | [Medium Precision Update](#medium-precision-update) |   |       |                                      |
///
///
/// ### Medium Precision Update
/// | Motion Data       | 32 bytes|||
/// |-------------------|---------|-------|--------------------------------------|
/// | position_x        | 2 bytes | [u16] | the x position of the object         |
/// | position_y        | 2 bytes | [u16] | the y position of the object         |
/// | position_z        | 2 bytes | [u16] | the z position of the object         |
/// | velocity_x        | 2 bytes | [u16] | the x velocity of the object         |
/// | velocity_y        | 2 bytes | [u16] | the y velocity of the object         |
/// | velocity_z        | 2 bytes | [u16] | the z velocity of the object         |
/// | acceleration_x    | 2 bytes | [u16] | the x acceleration of the object     |
/// | acceleration_y    | 2 bytes | [u16] | the y acceleration of the object     |
/// | acceleration_z    | 2 bytes | [u16] | the z acceleration of the object     |
/// | rotation_x        | 2 bytes | [u16] | the x rotation of the object as a quaternion |
/// | rotation_y        | 2 bytes | [u16] | the y rotation of the object as a quaternion |
/// | rotation_z        | 2 bytes | [u16] | the z rotation of the object as a quaternion |
/// | rotation_w        | 2 bytes | [u16] | the w rotation of the object as a quaternion |
/// | angular_velocity_x| 2 bytes | [u16] | the x angular velocity of the object |
/// | angular_velocity_y| 2 bytes | [u16] | the y angular velocity of the object |
/// | angular_velocity_z| 2 bytes | [u16] | the z angular velocity of the object |
///
/// ### Low Precision Update
/// | Motion Data       | 16 bytes|||
/// |-------------------|---------|-------|--------------------------------------|
/// | position_x        | 1 byte  | [u8] | the x position of the object         |
/// | position_y        | 1 byte  | [u8] | the y position of the object         |
/// | position_z        | 1 byte  | [u8] | the z position of the object         |
/// | velocity_x        | 1 byte  | [u8] | the x velocity of the object         |
/// | velocity_y        | 1 byte  | [u8] | the y velocity of the object         |
/// | velocity_z        | 1 byte  | [u8] | the z velocity of the object         |
/// | acceleration_x    | 1 byte  | [u8] | the x acceleration of the object     |
/// | acceleration_y    | 1 byte  | [u8] | the y acceleration of the object     |
/// | acceleration_z    | 1 byte  | [u8] | the z acceleration of the object     |
/// | rotation_x        | 1 byte  | [u8] | the x rotation of the object as a quaternion |
/// | rotation_y        | 1 byte  | [u8] | the y rotation of the object as a quaternion |
/// | rotation_z        | 1 byte  | [u8] | the z rotation of the object as a quaternion |
/// | rotation_w        | 1 byte  | [u8] | the w rotation of the object as a quaternion |
/// | angular_velocity_x| 1 byte  | [u8] | the x angular velocity of the object |
/// | angular_velocity_y| 1 byte  | [u8] | the y angular velocity of the object |
/// | angular_velocity_z| 1 byte  | [u8] | the z angular velocity of the object |
///
/// ## Primitive Geometry
///
/// ObjectData values to do with drawing primitive geometry in the scene
/// | Primitive Geometry ||||
/// |---------------|---------|-------|--------------------------------------|
/// | path_curve    | 1 byte  | [u8]  | The type of path the shape follows. 0x00 is a Line, 0x10 is a circle, etc |
/// | path_begin    | 2 bytes | [u16] | Start point of the path. Controls how much of the extrusion is used. |
/// | path_end      | 2 bytes | [u16] | end point of the path |
/// | path_scale_x  | 1 byte  | [u8]  | x scaling at the end of the extrusion. `128 means no scaling.|
/// | path_scale_y  | 1 byte  | [u8]  | y scaling at the end of the extrusion. `128 means no scaling.|
/// | path_scale_z  | 1 byte  | [u8]  | z scaling at the end of the extrusion. `128 means no scaling.|
/// | path_shear_x  | 1 byte  | [u8]  | x axis shear. 128 is no shear. |
/// | path_shear_y  | 1 byte  | [u8]  | y axis shear. 128 is no shear. |
/// | path_twist_end| 1 byte  | [i8]  | Twist applied at the end of the path. `128 is no twist.  |
/// | path_twist_begin| 1 byte  | [i8]  | Twist applied at the beginning of the path. `128 is no twist.  |
/// | path_radius_offset | 1 byte | [i8] | How much the shape's path moves away from the axis. 128 is no offset. |
/// | path_taper_y  | 1 byte  | [i8]  | tapers the shape from thick to thin on y axis. 128 is no taper. |
/// | path_taper_x  | 1 byte  | [i8]  | tapers the shape from thick to thin on x axis. 128 is no taper. |
/// | path_revolutions| 1 byte | [u8] | The number of times the shape revolves around its path. 128 is one revolution. |
/// | path_skew     | 1 byte  | [i8]  | Skews the shape along its path. 128 is no skew. |
/// | profile_curve | 1 byte  | [u8]  | What the shape looks like from profile. 0x00 is circular, 0x01 is square, etc |
/// | profile_begin | 2 bytes | [u16] | The start point of the profile. Controls how much extrusion is used horizontally. | \
/// | profile_end   | 2 bytes | [u16] | The end point of the profile horizontally |
/// | profile_hollow| 4 bytes | [f32] | Makes a hollow in the shape. EG. a hollow cylinder becomes a tube. |
///
///
/// ## Sound
///
/// ObjectData values to do with sounds the object may emit
/// | Sound ||||
/// |---------------|---------|-------|--------------------------------------|
/// | sound_id      | 16 bytes| [Uuid](uuid::Uuid) | The UUID of attached looped sounds |
/// | owner_id      | 16 bytes| [Uuid](uuid::Uuid) | UUID of the owner object. Null if there is no sound attached. |
/// | gain          | 4 bytes | [f32] | The gain of the attached sound |
/// | flags         | 1 byte  | [u8]  | Flags related to attached sound |
/// | radius        | 4 bytes | [f32] | Radius from the center of the object that the sound is audible from |
pub mod object_update;

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

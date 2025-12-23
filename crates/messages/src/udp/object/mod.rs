/// TODO: UNIMPLEMENTED
pub mod kill_object;
/// TODO: UNIMPLEMENTED
pub mod multiple_object_update;
/// # Object Update Cached
/// <https://wiki.secondlife.com/wiki/ObjectUpdateCached>
///
/// Send by the server to inform the viewer that it should begin rendering objects that may exist
/// in its cache. This is a very small packet that contains very minimal information, allowing
/// viewers to render objects quickly without there being a large amount of bandwidth overhead.
/// If the viewer finds these packets are not in the cache, a [`RequestMultipleObjects`] packet is
/// sent with the IDs missing in the cache, which will trigger the server to send object update
/// compressed packets.
///
/// ## Header
/// | ObjectUpdateCached        |             |                |                  |                 |
/// |---------------------------|-------------|----------------|------------------|-----------------|
/// | Packet Header             | id:14       | reliable: true | zerocoded: false |frequency: High  |
///
/// ## Packet Structure
/// | ObjectUpdateCached |    |       |                                          |
/// |---------------|---------|-------|------------------------------------------|
/// |region_handle  | 8 bytes | [u64] | The region ID of the objects |
/// |time_dilation  | 2 bytes | [u16] | the time dilation in the region |
/// |data_length    | 1 byte  | [u8]  | The count of objects in the packet |
/// |[cached_object_data](#cached_object_data)| variable byes| | Cached object data |
///
/// ## Cached Object Data
/// | Cached Object Data |    |       |                                          |
/// |---------------|---------|-------|------------------------------------------|
/// |id             | 4 bytes | [u32] | the region-local ID of the object        |
/// |crc            | 4 bytes | [u32] | the checksum generated to see if the object has changed |
/// |flags          | 4 bytes | [u32] | The flags pertaining to the object cache|
pub mod object_update_cached;

/// # Improved Terse Object Update
/// <https://wiki.secondlife.com/wiki/ImprovedTerseObjectUpdate>
///
/// Sent by the server to inform the viewer of changes in object position, velocity, acceleration
/// and other motion-related fields. Does not contain any information about how to render or create
/// the object.
///
/// ## Header
/// | ImprovedTerseObjectUpdate |             |                |                  |                 |
/// |---------------------------|-------------|----------------|------------------|-----------------|
/// | Packet Header             | id:15       | reliable: true | zerocoded: false |frequency: High  |
///
/// ## Packet Structure
/// | ImprovedTerseObjectUpdate| |       |                                          |
/// |---------------|---------|-------|------------------------------------------|
/// |region_handle  | 8 bytes | [u64] | The region ID of the objects |
/// |time_dilation  | 2 bytes | [u16] | the time dilation in the region |
/// |data_length    | 1 byte  | [u8]  | The count of objects in the packet |
/// |[terse_object_data](#terse_object_data)| variable byes| | object motion data |
///
/// ## Terse Object Data
/// | TerseObjectData| |       |                                          |
/// |---------------|---------|-------|------------------------------------------|
/// | local_id      | 4 bytes | [u32] | region local ID. used for most operations in lieu of the object's full UUID. |
/// | state         | 1 byte  | [u8]  | Unused except by grass to determine grass species |
/// | avatar        | 1 byte  | [bool]| flags the object as an avatar and not just an object|
/// | collision_plane_x| 4 or 0 bytes | [f32] | If the object is an avatar, read 4 bytes for the collision plane x. If not, read 0. |
/// | collision_plane_y| 4 or 0 bytes | [f32] | If the object is an avatar, read 4 bytes for the collision plane y. If not, read 0. |
/// | collision_plane_z| 4 or 0 bytes | [f32] | If the object is an avatar, read 4 bytes for the collision plane z. If not, read 0. |
/// | collision_plane_w| 4 or 0 bytes | [f32] | If the object is an avatar, read 4 bytes for the collision plane w. If not, read 0. |
/// | position_x    | 4 bytes | [f32] | x position of the object in the region |
/// | position_y    | 4 bytes | [f32] | y position of the object in the region |
/// | position_z    | 4 bytes | [f32] | z position of the object in the region |
/// | velocity_x    | 2 bytes | [u16] | x velocity of the object |
/// | velocity_y    | 2 bytes | [u16] | y velocity of the object |
/// | velocity_z    | 2 bytes | [u16] | z velocity of the object |
/// | acceleration_x| 2 bytes | [u16] | x acceleration of the object |
/// | acceleration_y| 2 bytes | [u16] | y acceleration of the object |
/// | acceleration_z| 2 bytes | [u16] | z acceleration of the object |
/// | rotation_x    | 2 bytes | [u16] | x rotation of the object |
/// | rotation_y    | 2 bytes | [u16] | y rotation of the object |
/// | rotation_z    | 2 bytes | [u16] | z rotation of the object |
/// | rotation_w    | 2 bytes | [u16] | w rotation of the object |
/// | angular_velocity_x| 2 bytes | [u16] | x angular velocity of the object |
/// | angular_velocity_y| 2 bytes | [u16] | y angular velocity of the object |
/// | angular_velocity_z| 2 bytes | [u16] | z angular velocity of the object |
///
pub mod improved_terse_object_update;

/// # Object Update Compressed
/// <https://wiki.secondlife.com/wiki/ObjectUpdateCompressed>
///
/// sent by the server in response to a RequestMultipleObjects packet. Contains a much more
/// conditional and abridged ObjectUpdate packet than the ObjectUpdate packet itself. though it may
/// look similar to an ObjectUpdate packet, many of the fields are optional, or out of order.
///
/// ## Header
/// | ObjectUpdateCompressed |             |                |                  |                 |
/// |------------------------|-------------|----------------|------------------|-----------------|
/// | Packet Header          | id:13       | reliable: true | zerocoded: false |frequency: High  |
///
/// ## Packet Structure
/// | ObjectUpdateCompressed| |       |                                          |
/// |---------------|---------|-------|------------------------------------------|
/// |region_handle  | 8 bytes | [u64] | The region ID of the objects |
/// |time_dilation  | 2 bytes | [u16] | the time dilation in the region |
/// |data_length    | 1 byte  | [u8]  | the count of objects in the packet|
/// |[compressed_object_data](#compressed_object-data)| variable byes| | Compressed object data |
///
/// ## Compressed Object Data
/// | Compressed Object Data  ||||
/// |---------------|---------|-------|--------------------------------------|
/// | update_flags  | 4 bytes | [u32] | flags determining the optional contents of the packet|
/// | data_size     | 2 bytes | [u16] | The number of bytes making up this object data block|
/// | full_id       | 16 bytes| [Uuid](uuid::Uuid) | The full UUID of the object |
/// | local_id      | 4 bytes | [u32] | region local ID. used for most operations in lieu of the object's full UUID. |
/// | pcode         | 1 byte  | [u8]  | Type of object represented by the packet. Avatar, grass, tree, etc |
/// | state         | 1 byte  | [u8]  | Unused except by grass to determine grass species |
/// | crc           | 4 bytes | [u32] | CRC values. Not currently checked against anything. |
/// | material      | 1 byte  | [u8]  | Type of material the object is made of. Wood, plastic, flesh, etc |
/// | click_action  | 1 byte  | [u8]  | The default action taken when the object is clicked. Open, sit, etc |
/// | scale_x       | 4 bytes | [f32] | The x value of the object's scale |
/// | scale_y       | 4 bytes | [f32] | The y value of the object's scale |
/// | scale_z       | 4 bytes | [f32] | the z value of the object's scale |
/// | position_x    | 4 bytes | [f32] | the x value of the object's position |
/// | position_y    | 4 bytes | [f32] | the y value of the object's position |
/// | position_z    | 4 bytes | [f32] | the z value of the object's position |
/// | rotation_z    | 4 bytes | [f32] | the z value of the object's rotation |
/// | rotation_z    | 4 bytes | [f32] | the z value of the object's rotation |
/// | rotation_z    | 4 bytes | [f32] | the z value of the object's rotation |
/// | owner_id      | 16 bytes or 0 | [Uuid](uuid::Uuid) | If the HasParticles, HasParticlesLegacy, or HasSound flags are present, read 16 bytes for the owner. Otherwise read 0.|
/// | angular_velocity_x| 4 bytes or 0| [f32] | If the HasAngularVelocity flag is present, read 4 bytes for the angular velocity x value. if not, read 0. |
/// | angular_velocity_y| 4 bytes or 0| [f32] | If the HasAngularVelocity flag is present, read 4 bytes for the angular velocity y value. if not, read 0. |
/// | angular_velocity_z| 4 bytes or 0| [f32] | If the HasAngularVelocity flag is present, read 4 bytes for the angular velocity z value. if not, read 0. |
/// | parent        | 4 bytes or 0 |[f32]| If the HasParent flag is present, read 4 bytes for the parent's scene-local ID. If not, read 0.|
/// | text_length   | 1 byte or 0 | [u8]  | If the HasText flag is present, read 1 byte for the text length. If not, read 0. |
/// | text          | variable bytes || If the HasText flag is present, read text_length bytes, to use as the object's hover text.|
/// | text_color_r  | 1 or 0 bytes| [u8]| If the HasText flag is present, read 1 byte for the text's red value. If not read 0.|
/// | text_color_g  | 1 or 0 bytes| [u8]| If the HasText flag is present, read 1 byte for the text's green value. If not read 0.|
/// | text_color_b  | 1 or 0 bytes| [u8]| If the HasText flag is present, read 1 byte for the text's blue value. If not read 0.|
/// | text_color_a  | 1 or 0 bytes| [u8]| If the HasText flag is present, read 1 byte for the text's alpha value. If not read 0.|
/// | media_length  | 1 byte or 0 bytes | [u8]  | If the MediaURL flag is present, read 1 byte for the media url length. If not, read 0.|
/// | media_url     | variable bytes || If the MediaURL flag is present, read media_length bytes, to use as the URL for any media attached to the object. Will always be a webpage. |
/// | particle_system_legacy_len| 1 byte or 0 bytes| [u8]| If the HasParticlesLegacy or HasParticles flags are set, read 1 byte for the particle system's length. If not, read 0. |
/// | particle_sytem_legacy| variable bytes || If the HasParticlesLegacy or HasParticles flags are set, read particle_system_len bytes for the particle system. If not, read 0. |
/// | [extra_params](#extra-params)| variable bytes || The first byte in the extra_params field is the number of extra params present in the object. If it is zero, read nothing. If it is not zero, see [extra_params](#extra-params)|
/// | sound_id      | 16 or 0 bytes| [Uuid](uuid::Uuid) | If the HasSound flag is present, read 16 bytes for the sound's UUID. If not, read 0. |
/// | gain          | 4 or 0 bytes | [f32] | If the HasSound flag is present, read 4 bytes for the gain. If not, read 0. |
/// | flags         | 1 or 0 byte  | [u8]  | If the HasSound flag is present, read 1 byte for the flags. If not, read 0.
/// | radius        | 4 or 0 bytes | [f32] | If the HasSound flag is present, read 4 bytes for the audible radius. If not, read 0. |
/// | name_value_length| 2 or 0 bytes| [u16] | if the HasNameValues flag is present, read 2 bytes for the name values length. If not, read 0. |
/// | name_value    | variable bytes || If the HasNameValues flag is present, read name_value_length bytes a the name_value. If not, read 0. |
/// | [primitive_geometry](#primitive-geometry) | 23 bytes | | Data for the viewer to draw primitive objects. Not optional, for some reason.|
/// | texture_entry_length| 16 bytes| [Uuid](uuid::Uuid) | The length of the texture entry data |
/// | texture_entry | variable bytes || Texture data for the object |
/// | texture_animation_length| 1 or 0 bytes |[u8]| If the TextureAnimation flag is present, read 1 byte for the texture animation length. If not, read 0. |
/// | texture_aimation | variable bytes || If the TextureAnimation flag is present, read texture_animation_length bytes for the texture_animation data. If not, read 0. |
/// | particle_system_length| 1 byte | [u8] | The length of the particle system data block |
/// | particle_sytem | variable bytes || The particle system information |
pub mod object_update_compressed;

/// # Request Multiple Objects
/// <https://wiki.secondlife.com/wiki/RequestMultipleObjects>
///
/// Sent by the client to request multiple objects from the server. Uses the local u32 ID of the
/// object
///
/// ## Header
/// | RequestMultipleObjects |             |                |                  |                 |
/// |------------------------|-------------|----------------|------------------|-----------------|
/// | Packet Header          | id:3        | reliable: true | zerocoded: true  |frequency: Medium|
///
/// ## Packet Structure
/// | RequestMultipleObjects| |       |                                          |
/// |---------------|---------|-------|------------------------------------------|
/// |agent_id        | 16 bytes| [Uuid](uuid::Uuid) | The full UUID of the object |
/// |session_id      | 16 bytes| [Uuid](uuid::Uuid) | The full UUID of the object |
/// |request_len     | 1 byte  | [u8] | The number of 5-byte requests in the packet|
/// |[object_data](#object-data)| variable byes|   | List of data to request     |
///
/// ## Object Data
/// | Object Data ||||
/// |---------------|---------|------|------------------|
/// |cache_miss_type| 1 byte  | [u8] | Type of data to request from the cache |
/// |id             | 4 bytes | [u32]| Local ID of the data to request |
pub mod request_multiple_objects;

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
/// | ObjectUpdate  ||||
/// |---------------|---------|-------|--------------------------------------|
/// | region_handle | 8 bytes | [u64] | region handle ID of the simulator |
/// | time_dilation | 2 bytes | [u16] | The current lag from the server. Used by physics simulations to keep up with real time. |
/// | offset        | 1 byte  |       | Unused byte                          |
/// | id            | 4 bytes | [u32] | region local ID. used for most operations in lieu of the object's full UUID. |
/// | state         | 1 byte  | [u8]  | Unused except by grass to determine grass species |
/// | full_id       | 16 bytes| [Uuid](uuid::Uuid) | The full UUID of the object |
/// | crc           | 4 bytes | [u32] | CRC values. Not currently checked against anything. |
/// | pcode         | 1 byte  | [u8]  | Type of object represented by the packet. Avatar, grass, tree, etc |
/// | material      | 1 byte  | [u8]  | Type of material the object is made of. Wood, plastic, flesh, etc |
/// | click_action  | 1 byte  | [u8]  | The default action taken when the object is clicked. Open, sit, etc |
/// | scale_x       | 4 bytes | [f32] | The x value of the object's scale |
/// | scale_y       | 4 bytes | [f32] | The y value of the object's scale |
/// | scale_z       | 4 bytes | [f32] | the z value of the object's scale |
/// | data_length   | 1 byte  | [u8]  | Number of bytes to read for the motion_data |
/// | [motion_data](#motion-data) | variable bytes | | Velocity, position and rotation values|
/// | parent_id     | 4 bytes | [u32] | Local ID of an object this object is a child of. 0 if none is present. |
/// | update_flags  | 4 bytes | [u32] | Gives various pieces of information to the viewer, like empty inventory or scripted |
/// | [primitive_geometry](#primitive-geometry) | 23 bytes | | Data for the viewer to draw primitive objects |
/// | texture_length| 2 bytes | [u16]  |  Number of bytes to read for the texture_entry data |
/// | texture_entry | variable bytes || Full property list of each object's face, including textures and colors. |
/// | anim_length   | 1 byte  | [u8]  | Number of bytes to read for the texture_anim_data |
/// | texture_anim  | variable bytes || Properties to set up texture animations of the face of the object |
/// | name_length   | 2 bytes | [u16] | Number of bytes to read for the name value. Big-endian |
/// | name_value    | variable bytes || Name value pairs specific to the object. Used for avatar names. |
/// | data_length   | 2 bytes | [u16] | Number of bytes to read for the generic appended data |
/// | data          | variable bytes || Generic appended data |
/// | text_length   | 2 bytes | [u16]  | Number of bytes to read for the text data |
/// | text          | variable bytes || Text that hovers over the object |
/// | text_color_r  | 1 or 0 bytes| [u8]| Hover text color's red val. If text_length is zero, don't read.|
/// | text_color_g  | 1 or 0 bytes| [u8]| Hover text color's green value. If text_length is zero, don't read.|
/// | text_color_b  | 1 or 0 bytes| [u8]| Hover text color's blue value. If text_length is zero, don't read.|
/// | text_color_a  | 1 or 0 bytes| [u8]| Hover text color's alpha value. If text_length is zero, don't read. |
/// | offset        | 0 or 3 bytes| [u8]| If text_color is not read, read 3 bytes of padding after the text_length.|
/// | media_length  | 1 byte  | [u8]  | Number of bytes to read for the media URL |
/// | media_url     | variable bytes || URL for any media attached to the object. Will always be a webpage. |
/// | particle_len  | 1 byte  | [u8]  | Number of bytes to read for the particle system data |
/// | particle_system | variable bytes || Attached particles for the object |
/// | extra_len     | 1 byte  | [u8]  | Number of bytes to read for the extra parameters |
/// | [extra_params](#extra-params)  | variable bytes || Data related to flexible primitives, sculpt data or light |
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
/// |----------|---------|-------|--------------------------------------|
/// | sound_id | 16 bytes| [Uuid](uuid::Uuid) | The UUID of attached looped sounds |
/// | owner_id | 16 bytes| [Uuid](uuid::Uuid) | UUID of the owner object. Null if there is no sound attached. |
/// | gain     | 4 bytes | [f32] | The gain of the attached sound |
/// | flags    | 1 byte  | [u8]  | Flags related to attached sound |
/// | radius   | 4 bytes | [f32] | Radius from the center of the object that the sound is audible from |
///
/// ## Extra Params
///
/// Extra parameters containing sculpt, light, flex, and other data.
/// | Extra Params ||||
/// |--------|---------|-------|--------------------------------------|
/// | extra_params_count| 1 byte | [u8] | Number of objects in the extra params field |
/// | tag    | 1 byte  |[u8]   | The type of parameter this is        |
/// | offset | 3 bytes |       | Three unused bytes of padding        |
/// | [sculpt_param](#sculpt_param), [flexi_param](#flexi_param), [light_param](#light_param), [projection_param](#projection_param),   [mesh_flags_param](#mesh_flags_param),   [reflection_probe_param](#reflection_probe_param), |variable bytes||Optional parameters describing various data. Each packet can contain multiple parameters. This is stored as a list.|
///
/// ## Sculpt Param
/// | Sculpt Param |        |                    |                                                                 |
/// |--------------|--------|--------------------|-----------------------------------------------------------------|
/// | texture_id | 16 bytes | [Uuid](uuid::Uuid) | The ID of the sculpt texture. This is also used as the mesh ID. |
/// | sculpt_type| 1 byte   | [u8]               | The type of the sculpt. 5 denotes a mesh.                       |
///
/// ## Flexi Param
/// TODO: UNIMPLEMENTED
///
/// ## Light Param
/// TODO: UNIMPLEMENTED
///
/// ## Projection Param
/// TODO: UNIMPLEMENTED
///
/// ## Mesh Flags Param
/// TODO: UNIMPLEMENTED
///
/// ## Materials Param
/// TODO: UNIMPLEMENTED
///
/// ## Reflection Probe Param
/// TODO:UNIMPLEMENTED
pub mod object_update;

mod util;

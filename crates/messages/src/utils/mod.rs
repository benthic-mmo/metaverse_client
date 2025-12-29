/// global values used for describing agent access levels
pub mod agent_access;
/// Global values used for describing item metadat, such as name, permissions, etc
pub mod item_metadata;
/// Material enum for defining object materials
/// used for adding textures
pub mod material;
/// Global values used for describing simulator object types
pub mod object_types;
/// Path data for defining how objects can be pulled, squashed and stretched.
pub mod path;
/// global values used for describing region capabilities
pub mod region_flags;
/// struct used to define renderable objects.
/// used to serialize retreived mesh data into a unified intermediate format, which can be used by
/// renderers to create graphics
pub mod render_data;
/// Skeleton object for defining the per-user skeleton
pub mod skeleton;
/// Sound data for playing sounds in-world
pub mod sound;
/// texture information for objects
pub mod texture_entry;

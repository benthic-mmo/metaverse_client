use glam::{Quat, Vec3};
use uuid::Uuid;

#[derive(Debug)]
pub struct GeneratorObject {
    pub full_id: Uuid,
    pub local_id: u32,
    pub parent_id: Option<u32>,
    pub position: Vec3,
    pub scale: Vec3,
    pub rotation: Quat,
}

#[derive(Debug, Clone)]
pub struct MinimalObjectUpdate<ExtraParams, TextureEntry, ObjectType> {
    /// Type of the object. Required for retrieving full data from the capability endpoint
    pub object_type: ObjectType,
    /// The full ID of the object
    pub full_id: Uuid,
    /// The scene local ID of the object
    pub local_id: u32,
    /// The position of the object.
    ///
    /// If the object is a child object, this position is relative to its parent object.
    pub position: Vec3,
    /// The rotation of the object.
    ///
    /// If the object is a child object, this is used to calculate the
    /// position
    pub rotation: Quat,
    /// The scale of the object.
    pub scale: Vec3,
    /// The local ID of the obeject's parent.
    pub parent: Option<u32>,
    /// The scene local ID of the object's parent.
    ///
    /// This is used to determine the scale Tposition and rotation if the object is part of a construction
    pub parent_id: Option<u32>,
    /// The name value of the object.
    ///
    /// This can encode extra data like attachment objects, or the avatar's name
    pub name_value: Option<String>,
    /// Extra parameters.
    ///
    /// Can contain definitions for things like sculpts (which include meshes), flexi data, light, and more.
    pub extra_params: Option<Vec<ExtraParams>>,

    /// Object's texture data
    pub texture: TextureEntry,

    /// CRC to enable cache invalidation
    pub crc: u32,

    pub region_id: String,
}

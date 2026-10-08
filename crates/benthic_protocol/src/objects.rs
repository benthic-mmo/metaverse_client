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

#[derive(Clone, Debug)]
pub struct MeshObjectData<TextureEntry, ObjectType> {
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
    pub rotation: Quat,
    /// The scale of the object.
    pub scale: Vec3,
    /// The local ID of the obeject's parent.
    pub parent: Option<u32>,
    /// The name value of the object.
    ///
    /// This can encode extra data like attachment objects, or the avatar's name
    pub name_value: Option<String>,

    /// Object's texture data
    pub texture: Option<TextureEntry>,

    pub sculpt_id: Uuid,

    /// CRC to enable cache invalidation
    pub crc: u32,

    pub region_id: String,
}

pub struct TreeObjectData {}

pub struct GrassObjectData {}

pub struct UnknownObject {}

pub struct ParticleObjectData {}

pub struct NewTreeObjectData {}

pub struct AttachmentObjectData {
    pub parent_id: Option<u32>,
}

pub struct ParametricPrimData {}

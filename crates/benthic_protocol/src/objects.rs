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
pub struct MeshObjectData<TextureEntries, ObjectType> {
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

    /// Object's texture data
    pub texture: Option<TextureEntries>,

    pub sculpt_id: Uuid,

    /// CRC to enable cache invalidation
    pub crc: u32,

    pub region_id: String,
}

#[derive(Debug, Clone)]
pub struct SculptObjectData<TextureEntries, SculptData> {
    pub full_id: Uuid,
    pub parent: Option<u32>,
    pub local_id: u32,
    pub position: Vec3,
    pub data: SculptData,
    pub rotation: Quat,
    pub scale: Vec3,
    pub texture: Option<TextureEntries>,
    pub crc: u32,
    pub region_id: String,
    pub retry_count: u32,
}

pub struct TreeObjectData {}

pub struct GrassObjectData {}

pub struct UnknownObject {}

pub struct ParticleObjectData {}

pub struct NewTreeObjectData {}

pub struct AttachmentObjectData {
    pub parent_id: Option<u32>,
}

#[derive(Clone, Debug)]
pub struct ParametricPrimData<TextureEntries, PrimPath> {
    pub full_id: Uuid,
    pub local_id: u32,
    pub scale: Vec3,
    pub position: Vec3,
    pub rotation: Quat,
    pub parent: Option<u32>,
    pub texture: Option<TextureEntries>,
    pub path_data: PrimPath,
    pub retry_count: u32,
}

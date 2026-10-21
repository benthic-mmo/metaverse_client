use crate::objects::types::TextureAnimation;
use glam::{Quat, Vec3};
use uuid::Uuid;

/// Trait for messages that are retriable.
pub trait RetriableData {
    /// How many times this object has been retried
    fn retry_count(&self) -> u32;
    /// A function to incrememnt the retry count for an object
    fn inc_retry_count(&mut self);
}

/// Common values for object instances
#[derive(Debug, Clone)]
pub struct ObjectInstanceInfo {
    /// full, globally unique UUID of the object's instance
    pub full_id: Uuid,
    /// scene-local u32 ID of the object's instance
    pub local_id: u32,
    /// scene-local u32 ID of the object's parent
    pub parent_id: Option<u32>,
    /// Position in space of the object
    pub position: Vec3,
    /// Scale of the object
    /// This scale is NOT relative to the parent. This is the global scale of the object.
    pub scale: Vec3,
    /// Rotation of the object
    pub rotation: Quat,
}

/// Data object for rendering parametric prims
#[derive(Clone, Debug)]
pub struct ParametricPrimData<TextureEntries, PrimPath> {
    /// Common values for object's instance
    pub object_info: ObjectInstanceInfo,
    /// per-face texture data
    pub texture: Option<TextureEntries>,
    /// Texture animation data
    pub texture_animation: Option<TextureAnimation>,
    /// Path data for drawing the parametric prim's path
    pub path_data: PrimPath,
    /// How many times this object has been retried
    pub retry_count: u32,
}
impl<T, P> RetriableData for ParametricPrimData<T, P> {
    fn retry_count(&self) -> u32 {
        self.retry_count
    }

    fn inc_retry_count(&mut self) {
        self.retry_count += 1;
    }
}

/// Data object for rendering sculpt objects
#[derive(Debug, Clone)]
pub struct SculptObjectData<TextureEntries, SculptData> {
    /// Common values for the object's instance
    pub object_info: ObjectInstanceInfo,
    /// data for building the sculpt object
    pub data: SculptData,
    /// per-face texture data
    pub texture: Option<TextureEntries>,
    /// texture animation data
    pub texture_animation: Option<TextureAnimation>,
    /// crc of the sculpt for cache invalidation
    pub crc: u32,
    /// region id where the sculpt was downloaded
    pub region_id: String,
    /// how many times this object has been retried
    pub retry_count: u32,
}
impl<T, P> RetriableData for SculptObjectData<T, P> {
    fn retry_count(&self) -> u32 {
        self.retry_count
    }

    fn inc_retry_count(&mut self) {
        self.retry_count += 1;
    }
}

///TODO: unimplemeted
pub struct AttachmentObjectData {
    /// ID of the object this is attached to
    pub parent_id: Option<u32>,
}

/// TODO: unimplemented
pub struct TreeObjectData {}

/// TODO: unimplemeted
pub struct GrassObjectData {}

/// TODO: unimplemented
pub struct UnknownObject {}

/// TODO: unimplemented
pub struct ParticleObjectData {}

/// TODO: unimplemented
pub struct NewTreeObjectData {}

use glam::{Quat, Vec3};
use uuid::Uuid;

use crate::objects::types::TextureAnimation;

pub trait RetriableData {
    fn retry_count(&self) -> u32;
    fn inc_retry_count(&mut self);
}

#[derive(Debug, Clone)]
pub struct ObjectInstanceInfo {
    pub full_id: Uuid,
    pub local_id: u32,
    pub parent_id: Option<u32>,
    pub position: Vec3,
    pub scale: Vec3,
    pub rotation: Quat,
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
    pub object_info: ObjectInstanceInfo,
    pub texture: Option<TextureEntries>,
    pub texture_animation: Option<TextureAnimation>,
    pub path_data: PrimPath,
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

#[derive(Debug, Clone)]
pub struct SculptObjectData<TextureEntries, SculptData> {
    pub object_info: ObjectInstanceInfo,
    pub data: SculptData,
    pub texture: Option<TextureEntries>,
    pub texture_animation: Option<TextureAnimation>,
    pub crc: u32,
    pub region_id: String,
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

use crate::{
    udp::object::object_update::ExtraParams,
    utils::{object_types::ObjectType, path::PrimPath, texture_entry::TextureEntries},
};
use benthic_protocol::objects::types::TextureAnimation;
use glam::{Quat, Vec3};
use uuid::Uuid;

/// Provides a common interface for object update packets.
/// this unifies the ObjectUpdate and ObjectUpdateCompressed packets.
pub trait ObjectUpdateData {
    /// Returns the type of the object
    fn object_type(&self) -> ObjectType;
    /// Returns the object's global id
    fn full_id(&self) -> Uuid;
    /// Returns the u32 scene-local parent ID.
    /// Objects without parents will return None
    fn parent_id(&self) -> Option<u32>;
    /// Returns the u32 scene-local ID
    fn local_id(&self) -> u32;
    /// Returns the name value string of the object.
    /// Not necessarily a name. This string can contain extra parameters used for attachments.
    fn name_value(&self) -> &Option<String>;
    /// Returns the position of the object in space
    fn position(&self) -> Vec3;
    /// Returns the rotation of the object
    fn rotation(&self) -> Quat;
    /// Returns the scale of the object
    fn scale(&self) -> Vec3;
    /// Returns te texture entry data of the object.
    /// Objects without textures will return None
    fn texture(&self) -> &Option<TextureEntries>;
    /// Returns the crc of the object
    fn crc(&self) -> u32;
    /// Returns the extra parameter data of the object
    /// Objects without extra params will return None
    fn extra_params(&self) -> Option<&[ExtraParams]>;
    /// Returns the sculpt path of the object
    /// Objects without sculpt information will return None
    fn sculpt_path(&self) -> Option<PrimPath>;
    /// Returns the texture animation data of the object
    /// Objects without texture animation data will return None
    fn texture_animation(&self) -> Option<TextureAnimation>;
}

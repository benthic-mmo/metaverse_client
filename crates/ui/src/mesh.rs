use bevy::{
    animation::AnimatedBy, camera::visibility::DynamicSkinnedMeshBounds,
    mesh::skinning::SkinnedMesh, prelude::*,
};
use bevy_gltf::{GltfMaterialName, GltfMeshName, GltfSceneName};

pub struct BenthicMeshPlugin;
impl Plugin for BenthicMeshPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<GltfSceneName>()
            .register_type::<Transform>()
            .register_type::<GlobalTransform>()
            .register_type::<TransformTreeChanged>()
            .register_type::<Children>()
            .register_type::<Visibility>()
            .register_type::<ChildOf>()
            .register_type::<InheritedVisibility>()
            .register_type::<ViewVisibility>()
            .register_type::<AnimationPlayer>()
            .register_type::<Name>()
            .register_type::<Mesh3d>()
            .register_type::<bevy::camera::primitives::Aabb>()
            .register_type::<SkinnedMesh>()
            .register_type::<GltfMeshName>()
            .register_type::<GltfMaterialName>()
            .register_type::<AnimatedBy>()
            .register_type::<DynamicSkinnedMeshBounds>();
    }
}

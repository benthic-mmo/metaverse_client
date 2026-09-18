use bevy::camera::Camera3dDepthLoadOp;
use bevy::core_pipeline::oit::OrderIndependentTransparencySettings;
use bevy::light::{AmbientLight, DirectionalLight, PointLight};
use bevy::prelude::*;
use bevy::render::view::{ColorGradingGlobal, ColorGradingSection};
use bevy_gizmos::config::GizmoConfigStore;
use bevy_inspector_egui::quick::WorldInspectorPlugin;

pub struct InspectorPlugin;

impl Plugin for InspectorPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<ColorGradingSection>()
            .register_type::<ColorGradingGlobal>()
            .register_type::<AmbientLight>()
            .register_type::<PointLight>()
            .register_type::<DirectionalLight>()
            .register_type::<Camera3dDepthLoadOp>()
            .register_type::<OrderIndependentTransparencySettings>()
            .register_type::<GizmoConfigStore>()
            .add_plugins(WorldInspectorPlugin::new());
    }
}

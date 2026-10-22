use std::path::PathBuf;
mod common;
use crate::common::{
    bevy::{
        DebugVisibility, draw_skeleton_debug, handle_ui_buttons, setup, update_mesh_visibility,
    },
    mock_avatar_load,
};
use bevy::{
    DefaultPlugins,
    app::{App, Startup, Update},
    asset::{AssetMode, AssetPlugin, UnapprovedPathMode},
    prelude::PluginGroup,
    winit::WinitPlugin,
};
use bevy_panorbit_camera::PanOrbitCameraPlugin;

/// this test debugs the entire pipeline from the very beginning.
/// Input data is the SceneObject and Mesh data coming directly from the server.
/// this is meant to create an easy debugging setup for each step of the process to find bugs in
/// mesh handling.
#[tokio::test]
async fn test_mesh_generation() {
    let _artifacts = mock_avatar_load("puffball".to_string(), 4).await;
}

#[tokio::test]
async fn display_humanoid_puffball() {
    display_test_model_for("humanoid_puffball", 3).await;
}

/// this test builds the model and displays using Bevy.
#[tokio::test]
async fn display_test_model() {
    display_test_model_for("puffball", 4).await;
}

/// Builds and displays an avatar model using Bevy.
async fn display_test_model_for(avatar_name: &str, outfit_size: usize) {
    let artifacts = mock_avatar_load(avatar_name.to_string(), outfit_size).await;

    let tests_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join(format!("tests/generated/{}", common::AGENT_ID));

    let mut app = App::new();
    app.add_plugins(
        DefaultPlugins
            .set(WinitPlugin {
                run_on_any_thread: true,
            })
            .set(AssetPlugin {
                file_path: tests_dir.to_string_lossy().into_owned(),
                mode: AssetMode::Unprocessed,
                unapproved_path_mode: UnapprovedPathMode::Allow,
                ..Default::default()
            }),
    );

    app.add_plugins(PanOrbitCameraPlugin);
    app.insert_resource(DebugVisibility {
        show_mesh: true,
        show_skeleton: true,
        show_joint_axes: false,
    });
    app.insert_resource(artifacts);
    app.add_systems(Startup, setup);
    app.add_systems(Update, handle_ui_buttons);
    app.add_systems(Update, update_mesh_visibility);
    app.add_systems(Update, draw_skeleton_debug);
    app.run();
}

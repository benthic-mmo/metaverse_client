use std::path::PathBuf;

use bevy::{
    DefaultPlugins,
    app::{App, Startup, Update},
    asset::{AssetMode, AssetPlugin, AssetServer, UnapprovedPathMode},
    ecs::{component::Component, system::Commands},
    math::VectorSpace,
    prelude::*,
    winit::WinitPlugin,
};
use bevy_panorbit_camera::{PanOrbitCamera, PanOrbitCameraPlugin};

#[tokio::test]
async fn display_test_gltf() {
    let tests_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/data");

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
                ..default()
            }),
    );

    app.add_plugins((PanOrbitCameraPlugin,));

    app.add_systems(Startup, setup_test_model);
    app.add_systems(Update, (handle_test_click, scale_test_model));

    app.run();
}

#[derive(Component)]
struct TestModel;

fn setup_test_model(mut commands: Commands, asset_server: Res<AssetServer>) {
    let scene = asset_server.load(GltfAssetLabel::Scene(0).from_asset("osgrid.glb"));
    let scale = Vec3::new(127.02799, 128.055, 21.130907);
    let rotation = Quat::from_xyzw(0.0, 0.0, 0.0, 0.0);
    let position = Vec3::ZERO;

    let transform = Transform {
        translation: position,
        rotation,
        scale,
    };

    commands.spawn((
        WorldAssetRoot(scene),
        transform,
        Visibility::Visible,
        Pickable::default(),
        TestModel,
        Name::new("TestModel"),
    ));
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(4.0, 4.0, 6.0).looking_at(Vec3::ZERO, Vec3::Y),
        PanOrbitCamera {
            radius: Some(5.0),
            ..default()
        },
    ));

    commands.spawn((
        DirectionalLight {
            illuminance: 10_000.0,
            ..default()
        },
        Transform::from_xyz(4.0, 6.0, 4.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));
}

fn handle_test_click(mut events: MessageReader<Pointer<Click>>) {
    for event in events.read() {
        println!("CLICKED {:?}", event.entity);
    }
}

fn scale_test_model(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut query: Query<&mut Transform, With<TestModel>>,
) {
    let Ok(mut transform) = query.single_mut() else {
        return;
    };

    if keyboard.pressed(KeyCode::ArrowUp) {
        transform.scale *= 1.01;
    }

    if keyboard.pressed(KeyCode::ArrowDown) {
        transform.scale *= 0.99;
    }
}

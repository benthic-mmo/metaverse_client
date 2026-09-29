mod common;

use benthic_protocol::messages::ui::land_update::{LandData, LandUpdate};
use bevy::{
    DefaultPlugins,
    app::{App, PluginGroup, Startup},
    asset::{AssetMode, AssetPlugin, RenderAssetUsages, UnapprovedPathMode},
    ecs::{message::MessageWriter, resource::Resource},
    prelude::*,
    winit::WinitPlugin,
};
use bevy_panorbit_camera::{PanOrbitCamera, PanOrbitCameraPlugin};
use common::mock_session;
use metaverse_environment::layer_handler::handle_layer;
use metaverse_messages::packet::{packet_protocol::Packet, packet_types::PacketType};
use std::path::PathBuf;

#[tokio::test]
async fn build_land() {
    let mut session = mock_session();
    let data_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/data");

    for entry in std::fs::read_dir(data_dir.clone()).unwrap() {
        let path = entry.unwrap().path();

        if !path.is_file() {
            continue;
        }

        let buf = std::fs::read(&path).unwrap();

        let packet = Packet::from_bytes(&buf)
            .unwrap_or_else(|e| panic!("failed to parse {:?}: {:?}", path, e));

        match packet.body {
            PacketType::LayerData(packet) => {
                handle_layer(*packet, &mut session).unwrap();
            }
            other => {
                panic!("expected LandPacket for {:?}, got {:?}", path, other);
            }
        }
    }

    display_land(session.share_dir_root.join("land"));
}

#[derive(Resource)]
struct LandPath(PathBuf);

fn display_land(land_path: PathBuf) {
    let mut app = App::new();
    app.add_plugins(
        DefaultPlugins
            .set(WinitPlugin {
                run_on_any_thread: true,
            })
            .set(AssetPlugin {
                file_path: land_path.to_string_lossy().into_owned(),
                mode: AssetMode::Unprocessed,
                unapproved_path_mode: UnapprovedPathMode::Allow,
                ..Default::default()
            }),
    )
    .add_plugins(PanOrbitCameraPlugin)
    .insert_resource(LandPath(land_path.clone()))
    .add_systems(Startup, setup)
    .add_systems(Startup, render_land)
    .run();
}

pub fn setup(mut commands: Commands) {
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, 500.0, 500.0).looking_at(Vec3::ZERO, Vec3::Y),
        PanOrbitCamera {
            focus: Vec3::new(0.0, 1.0, 0.0),
            ..Default::default()
        },
        AmbientLight {
            color: Color::WHITE,
            brightness: 500.0,
            affects_lightmapped_meshes: true,
        },
    ));
}

fn render_land(
    land_path: Res<LandPath>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let Ok(entries) = std::fs::read_dir(&land_path.0) else {
        error!("land dir missing: {}", land_path.0.display());
        return;
    };

    for entry in entries.flatten() {
        let path = entry.path();

        if !path.is_file() {
            continue;
        }

        let Ok(json_str) = std::fs::read_to_string(&path) else {
            error!("Failed to read file {}", path.display());
            continue;
        };

        let Ok(land_data) = serde_json::from_str::<LandData>(&json_str) else {
            error!("Failed to deserialize JSON: {}", path.display());
            continue;
        };

        info!(
            "Rendering {}: {} vertices, {} indices, position {:?}",
            path.display(),
            land_data.vertices.len(),
            land_data.indices.len(),
            land_data.position
        );

        let mut mesh = Mesh::new(
            bevy::mesh::PrimitiveTopology::TriangleList,
            RenderAssetUsages::RENDER_WORLD,
        );

        mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, land_data.vertices);
        mesh.insert_indices(bevy::mesh::Indices::U16(land_data.indices));
        mesh.compute_smooth_normals();

        let mesh_handle = meshes.add(mesh);

        let material = materials.add(StandardMaterial {
            base_color: Color::WHITE,
            ..default()
        });

        commands.spawn((
            Mesh3d(mesh_handle),
            MeshMaterial3d(material),
            Transform::from_translation(land_data.position),
        ));
    }
}

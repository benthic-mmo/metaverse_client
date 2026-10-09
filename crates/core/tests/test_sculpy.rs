mod common;

use benthic_protocol::objects::SculptObjectData;
use bevy::{
    DefaultPlugins,
    app::{App, Startup, Update},
    asset::UnapprovedPathMode,
    prelude::*,
    winit::WinitPlugin,
};
use bevy_panorbit_camera::{PanOrbitCamera, PanOrbitCameraPlugin};
use glam::Vec3;
use metaverse_messages::{
    http::scene::SculptType,
    packet::{packet_protocol::Packet, packet_types::PacketType},
    udp::object::{object_update::ExtraParams, util::ObjectUpdateData},
};
use metaverse_objects::{object_updates::ObjectUpdateAction, sculpt_objects::handle_sculpt_object};
use std::path::PathBuf;

use crate::common::mock_session;

#[tokio::test]
async fn test_sculpys() {
    let generated_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/generated");
    let data_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/data/sculpt_debug");

    let _session = mock_session(generated_dir.clone()).await;
    let mut objects: Vec<PathBuf> = Vec::new();

    for entry in std::fs::read_dir(&data_dir).unwrap() {
        let path = entry.unwrap().path();

        if path.extension().is_none_or(|ext| ext != "bin") {
            continue;
        }

        let buffer = std::fs::read(&path).unwrap();
        let packet = Packet::from_bytes(&buffer)
            .unwrap_or_else(|e| panic!("Failed to parse {}: {e}", path.display()));

        let object_updates: Vec<Box<dyn ObjectUpdateData>> = match packet.body {
            PacketType::ObjectUpdateCompressed(data) => data
                .object_data
                .into_iter()
                .map(|object| Box::new(object) as Box<dyn ObjectUpdateData>)
                .collect(),

            PacketType::ObjectUpdate(data) => {
                vec![Box::new(*data) as Box<dyn ObjectUpdateData>]
            }

            _ => panic!("{} is not an object update packet", path.display()),
        };

        for object in &object_updates {
            let Some(params) = object.extra_params() else {
                continue;
            };

            let sculpt_type = params.iter().find_map(|param| match param {
                ExtraParams::Sculpt(sculpt) => Some(sculpt.sculpt_type),
                _ => None,
            });

            let Some(sculpt_type) = sculpt_type else {
                continue;
            };

            if matches!(sculpt_type, SculptType::Mesh | SculptType::Unknown) {
                continue;
            }

            for param in params {
                let ExtraParams::Sculpt(sculpt) = param else {
                    continue;
                };

                let texture_path = data_dir
                    .join("textures")
                    .join(format!("{}.png", sculpt.texture_id));

                let sculpt_object = SculptObjectData {
                    full_id: object.full_id(),
                    parent: object.parent_id(),
                    local_id: object.local_id(),
                    position: object.position(),
                    data: sculpt.clone(),
                    rotation: object.rotation(),
                    scale: object.scale(),
                    texture: object.texture().clone(),
                    crc: object.crc(),
                    region_id: "".to_string(),
                    retry_count: 0,
                };
                let actions =
                    match handle_sculpt_object(&generated_dir, texture_path, sculpt_object).await {
                        Ok(actions) => actions,
                        Err(e) => {
                            eprintln!(
                                "Failed to process sculpt {} from {}: {e}",
                                sculpt.texture_id,
                                path.display()
                            );
                            continue;
                        }
                    };
                for action in actions {
                    match action {
                        ObjectUpdateAction::Render(prim_data) => {
                            if let Some(mesh_path) = prim_data.mesh_path {
                                objects.push(mesh_path);
                            }
                        }
                        _ => panic!("Not a render parametric prim object update"),
                    }
                }
            }
        }
    }

    assert!(!objects.is_empty(), "No sculpt vertices were generated");

    println!("Displaying {} sculpt objects", objects.len());

    display_gltf(objects);
}

#[derive(Resource)]
struct GltfQueue {
    paths: Vec<PathBuf>,
    index: usize,
    current_entity: Option<Entity>,
}

#[derive(Component)]
struct NextObjectButton;

fn display_gltf(paths: Vec<PathBuf>) {
    let mut app = App::new();
    let tests_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join(format!("tests/generated/{}", common::AGENT_ID));
    app.add_plugins(
        DefaultPlugins
            .set(WinitPlugin {
                run_on_any_thread: true,
            })
            .set(AssetPlugin {
                file_path: tests_dir.into_string().unwrap(),
                mode: AssetMode::Unprocessed,
                unapproved_path_mode: UnapprovedPathMode::Allow,
                ..Default::default()
            }),
    );

    app.add_plugins(PanOrbitCameraPlugin);

    app.insert_resource(GltfQueue {
        paths,
        index: 0,
        current_entity: None,
    });

    app.add_systems(Startup, setup_gltf_viewer);
    app.add_systems(Update, next_object_button);

    app.run();
}

fn setup_gltf_viewer(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut queue: ResMut<GltfQueue>,
) {
    // Spawn the first object.
    let first_path = queue.paths[0].to_string_lossy().to_string();

    let entity = commands
        .spawn((
            WorldAssetRoot(asset_server.load(GltfAssetLabel::Scene(0).from_asset(first_path))),
            Transform::default(),
        ))
        .id();

    queue.current_entity = Some(entity);

    // Camera.
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, -3.0, 1.5).looking_at(Vec3::ZERO, Vec3::Z),
        PanOrbitCamera {
            focus: Vec3::ZERO,
            radius: Some(3.0),
            ..default()
        },
    ));

    // Lighting.
    commands.spawn((
        PointLight {
            intensity: 5000.0,
            ..default()
        },
        Transform::from_xyz(2.0, 2.0, 4.0),
    ));

    // Next Object button.
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(20.0),
                left: Val::Px(20.0),
                width: Val::Px(180.0),
                height: Val::Px(50.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(Color::srgb(0.15, 0.15, 0.15)),
            Button,
            NextObjectButton,
        ))
        .with_child((
            Text::new(format!("Next Object (1/{})", queue.paths.len())),
            TextFont {
                font_size: FontSize::Px(12.0),
                ..default()
            },
            TextColor(Color::WHITE),
        ));
}

fn next_object_button(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut queue: ResMut<GltfQueue>,
    buttons: Query<&Interaction, (Changed<Interaction>, With<NextObjectButton>)>,
    mut button_text: Query<&mut Text, With<NextObjectButton>>,
) {
    for interaction in &buttons {
        if *interaction != Interaction::Pressed {
            continue;
        }

        // Remove the previous GLTF scene root.
        if let Some(entity) = queue.current_entity.take() {
            commands.entity(entity).despawn();
        }

        // Advance, wrapping back to the first object.
        queue.index = (queue.index + 1) % queue.paths.len();

        let path = queue.paths[queue.index].to_string_lossy().to_string();

        let entity = commands
            .spawn((
                WorldAssetRoot(asset_server.load(GltfAssetLabel::Scene(0).from_asset(path))),
                Transform::default(),
            ))
            .id();

        queue.current_entity = Some(entity);

        // Update the button label.
        for mut text in &mut button_text {
            **text = format!("Next Object ({}/{})", queue.index + 1, queue.paths.len());
        }

        println!(
            "Displaying object {}/{}: {}",
            queue.index + 1,
            queue.paths.len(),
            queue.paths[queue.index].display()
        );
    }
}

use bevy::prelude::*;
use bevy_panorbit_camera::PanOrbitCamera;
use metaverse_messages::ui::custom::layer_update::LayerUpdate;

#[derive(Event)]
pub struct LayerUpdateEvent {
    pub value: LayerUpdate,
}

#[derive(Resource)]
pub struct PendingLayer {
    pub handle: Handle<Gltf>,
    pub position: Vec3,
}

pub fn setup_environment(mut commands: Commands) {
    commands.spawn((
        PointLight {
            shadows_enabled: true,
            ..default()
        },
        Transform::from_xyz(-54.0, 297.0, -33.0),
    ));
    commands.spawn((
        Transform {
            translation: Vec3::new(-54.91, 297.45, -33.25),
            rotation: Quat::from_xyzw(0.24934214, 0.7405695, 0.44497907, -0.43163628),
            ..default()
        },
        PanOrbitCamera::default(),
    ));
}

pub fn handle_layer_update(
    mut ev_layer_update: EventReader<LayerUpdateEvent>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    let factor = 4;
    for layer_update in ev_layer_update.read() {
        println!("Placing layer at {:?}", layer_update.value.position);
        let x = layer_update.value.position.x * factor;
        let y = layer_update.value.position.y * factor;
        let handle: Handle<Gltf> = asset_server.load(layer_update.value.path.clone());
        commands.insert_resource(PendingLayer {
            handle,
            position: Vec3::new(x as f32, 0.0, y as f32),
        });
    }
}

pub fn check_model_loaded(
    mut commands: Commands,
    pending: Option<Res<PendingLayer>>,
    layer_assets: Res<Assets<Gltf>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    if let Some(pending_layer) = pending {
        if let Some(gltf) = layer_assets.get(&pending_layer.handle) {
            let white_material = materials.add(StandardMaterial {
                base_color: Color::WHITE,
                ..Default::default()
            });
            commands.spawn((
                SceneRoot(gltf.scenes[0].clone()),
                Transform::from_translation(pending_layer.position),
                MeshMaterial3d::from(white_material.clone()),
            ));
            commands.remove_resource::<PendingLayer>();
        }
    }
}

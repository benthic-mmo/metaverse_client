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

#[derive(Resource)]
pub struct PendingLayers {
    pub items: Vec<PendingLayer>,
}

pub fn setup_environment(mut commands: Commands) {
    commands.spawn((
        PointLight {
            shadows_enabled: false,
            intensity: 3000.0,
            range: 4000.0,
            ..default()
        },
        Transform::from_xyz(0.0, 20.0, 0.0),
    ));
    commands.spawn((
        Transform {
            translation: Vec3::new(678.0, 471.0, 962.0),
            ..default()
        },
        PanOrbitCamera::default(),
    ));
}

pub fn handle_layer_update(
    mut ev_layer_update: EventReader<LayerUpdateEvent>,
    mut pending_layers: ResMut<PendingLayers>,
    asset_server: Res<AssetServer>,
) {
    let factor = 16;
    for layer_update in ev_layer_update.read() {
        let x = layer_update.value.position.x * factor;
        let y = layer_update.value.position.y * factor;
        let handle: Handle<Gltf> = asset_server.load(layer_update.value.path.clone());
        pending_layers.items.push(PendingLayer {
            handle,
            position: Vec3::new(x as f32, 0.0, y as f32),
        });
    }
}

pub fn check_model_loaded(
    mut commands: Commands,
    mut pending_layers: ResMut<PendingLayers>,
    layer_assets: Res<Assets<Gltf>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let mut ready = vec![];
    for (i, layer) in pending_layers.items.iter().enumerate(){
        if let Some(gltf) = layer_assets.get(&layer.handle) {
            let white_material = materials.add(StandardMaterial {
                base_color: Color::WHITE,
                ..Default::default()
            });
            commands.spawn((
                SceneRoot(gltf.scenes[0].clone()),
                Transform::from_translation(layer.position),
                MeshMaterial3d::from(white_material.clone()),
            ));
            ready.push(i)
        }
    }
    for i in ready.iter().rev() {
        pending_layers.items.remove(*i);
    }
}

pub fn _log_camera_position_system(query: Query<&Transform, With<Camera>>) {
    for transform in query.iter() {
        println!("Camera position: {:?}", transform.translation);
    }
}

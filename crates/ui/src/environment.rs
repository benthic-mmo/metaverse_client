use bevy::asset::RenderAssetUsages;
use bevy::light::SunDisk;
use bevy::prelude::*;
use bevy_panorbit_camera::PanOrbitCamera;
use metaverse_messages::ui::land_update::{LandData, LandUpdate};
use metaverse_messages::ui::mesh_update::MeshType;
use metaverse_messages::ui::skybox_update::SkyboxUpdate;
use metaverse_messages::ui::water_update::WaterUpdate;
use std::fs;

use crate::render::{MainCamera, MeshQueue, Renderable, RenderableHandle};

#[derive(Message)]
pub struct WaterUpdateEvent {
    pub value: WaterUpdate,
}

#[derive(Message)]
pub struct SkyboxUpdateEvent {
    pub value: SkyboxUpdate,
}

#[derive(Message)]
pub struct LandUpdateEvent {
    pub value: LandUpdate,
}

#[derive(Component)]
pub struct WaterPlane;

#[derive(Component)]
pub struct SunLight;

#[derive(Resource)]
pub struct SunState {
    pub current_phase: f32,
    pub target_phase: f32,
}

pub fn setup_environment(mut commands: Commands) {
    commands.spawn((
        DirectionalLight {
            shadows_enabled: true,
            illuminance: 10000.0,
            ..default()
        },
        SunDisk::default(),
        Transform::from_xyz(200.0, 100.0, 100.0),
        SunLight,
    ));
    commands.spawn((
        PanOrbitCamera {
            radius: Some(2.0),
            ..Default::default()
        },
        MainCamera,
    ));
}

pub fn handle_land_update(
    mut ev_land_update: MessageReader<LandUpdateEvent>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut mesh_queue: ResMut<MeshQueue>,
) {
    for patch in ev_land_update.read() {
        let land = &patch.value;
        match fs::read_to_string(&land.path) {
            Ok(json_str) => match serde_json::from_str::<LandData>(&json_str) {
                Ok(land_data) => {
                    let mut mesh = Mesh::new(
                        bevy::mesh::PrimitiveTopology::TriangleList,
                        RenderAssetUsages::RENDER_WORLD,
                    );
                    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, land_data.vertices);
                    mesh.insert_indices(bevy::mesh::Indices::U16(land_data.indices));
                    mesh.compute_smooth_normals();
                    let mesh_handle = meshes.add(mesh);
                    mesh_queue.pending.push(Renderable {
                        handle: RenderableHandle::Mesh(mesh_handle),
                        transform: Transform {
                            translation: land_data.position,
                            ..Default::default()
                        },
                        parent: None,
                        mesh_type: MeshType::Land,
                        id: None,
                    })
                }
                Err(err) => error!("Failed to deserialize JSON: {}", err),
            },
            Err(err) => error!("Failed to read file {}", err),
        }
    }
}

pub fn handle_water_update(
    mut ev_water_update: MessageReader<WaterUpdateEvent>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    for update in ev_water_update.read() {
        let mesh = meshes.add(Plane3d::default().mesh().size(2000.0, 2000.0));

        let water_color = update.value.color;
        let material = materials.add(StandardMaterial {
            base_color: Color::srgba(
                water_color.r / 255.0,
                water_color.g / 255.0,
                water_color.b / 255.0,
                water_color.a / 255.0,
            ),
            metallic: 0.0,
            perceptual_roughness: 0.2,
            ..default()
        });

        commands.spawn((
            WaterPlane,
            Mesh3d(mesh),
            MeshMaterial3d(material),
            Transform::from_xyz(0.0, update.value.height, 0.0),
        ));
    }
}

pub fn handle_skybox_update(
    mut ev_skybox_update: MessageReader<SkyboxUpdateEvent>,
    mut sun: ResMut<SunState>,
) {
    for update in ev_skybox_update.read() {
        sun.target_phase = update.value.sun_phase as f32;
    }
}

pub fn update_sun(
    time: Res<Time>,
    mut sun: ResMut<SunState>,
    mut query: Query<&mut Transform, With<SunLight>>,
) {
    let dt = time.delta_secs();
    let speed = 5.0;

    sun.current_phase = sun
        .current_phase
        .lerp(sun.target_phase, 1.0 - (-speed * dt).exp());

    let dir = Vec3::new(sun.current_phase.cos(), sun.current_phase.sin(), 0.3).normalize();

    for mut transform in &mut query {
        transform.rotation = Quat::from_rotation_arc(Vec3::NEG_Z, dir);
    }
}

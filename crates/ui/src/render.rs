use bevy::asset::RenderAssetUsages;
use bevy::platform::collections::HashMap;
use bevy::prelude::*;
use bevy_panorbit_camera::PanOrbitCamera;
use metaverse_messages::ui::land_update::{LandData, LandUpdate};
use metaverse_messages::ui::mesh_update::{MeshType, MeshUpdate};
use std::fs;

use crate::textures::environment::HeightMaterial;

#[derive(Resource)]
pub struct SceneIDMap {
    pub entities: HashMap<u32, Entity>,
}

#[derive(Message)]
pub struct MeshUpdateEvent {
    pub value: MeshUpdate,
}

#[derive(Message)]
pub struct LandUpdateEvent {
    pub value: LandUpdate,
}

pub enum RenderableHandle {
    Gltf(Handle<Gltf>),
    Mesh(Handle<Mesh>),
}

#[derive(Resource)]
pub struct Renderable {
    pub handle: RenderableHandle,
    pub position: Vec3,
    pub scale: Vec3,
    pub rotation: Quat,
    pub parent: Option<u32>,
    pub scene_id: Option<u32>,
    pub mesh_type: MeshType,
}

#[derive(Resource)]
pub struct MeshQueue {
    pub items: Vec<Renderable>,
}

pub fn setup_environment(mut commands: Commands) {
    commands.spawn((
        PointLight {
            shadows_enabled: true,
            intensity: 100000000.0,
            range: 1000.0,
            ..default()
        },
        Transform::from_xyz(200.0, 100.0, 100.0),
    ));
    commands.spawn((
        Transform {
            translation: Vec3::new(678.0, 471.0, 962.0),
            ..default()
        },
        PanOrbitCamera::default(),
    ));
}

pub fn handle_land_update(
    mut ev_land_update: MessageReader<LandUpdateEvent>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut mesh_queue: ResMut<MeshQueue>,
) {
    for patch in ev_land_update.read() {
        let path = &patch.value; // path to JSON file
        match fs::read_to_string(&path.path) {
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
                    mesh_queue.items.push(Renderable {
                        handle: RenderableHandle::Mesh(mesh_handle),
                        scale: Vec3::ONE,
                        rotation: Quat::IDENTITY,
                        scene_id: None,
                        parent: None,
                        position: land_data.position,
                        mesh_type: MeshType::Land,
                    })
                }
                Err(err) => error!("Failed to deserialize JSON: {}", err),
            },
            Err(err) => error!("Failed to read file {}", err),
        }
    }
}

pub fn handle_mesh_update(
    mut ev_mesh_update: MessageReader<MeshUpdateEvent>,
    mut mesh_queue: ResMut<MeshQueue>,
    asset_server: Res<AssetServer>,
) {
    for renderable in ev_mesh_update.read() {
        let handle: Handle<Gltf> = asset_server.load(renderable.value.path.clone());
        mesh_queue.items.push(Renderable {
            handle: RenderableHandle::Gltf(handle),
            scale: renderable.value.scale,
            rotation: renderable.value.rotation,
            position: renderable.value.position,
            scene_id: renderable.value.scene_id,
            parent: renderable.value.parent,
            mesh_type: renderable.value.mesh_type.clone(),
        });
    }
}

pub fn check_model_loaded(
    mut commands: Commands,
    mut mesh_queue: ResMut<MeshQueue>,
    mut id_map: ResMut<SceneIDMap>,
    gltf_assets: Res<Assets<Gltf>>,
    mut standard_materials: ResMut<Assets<StandardMaterial>>,
    mut height_materials: ResMut<Assets<HeightMaterial>>,
    asset_server: Res<AssetServer>,
) {
    let mut ready = vec![];
    for (i, layer) in mesh_queue.items.iter().enumerate() {
        match &layer.handle {
            RenderableHandle::Gltf(gltf_handle) => {
                if let Some(gltf) = gltf_assets.get(gltf_handle) {
                    let entity = commands
                        .spawn((
                            SceneRoot(gltf.scenes[0].clone()),
                            Transform {
                                translation: layer.position,
                                scale: layer.scale,
                                rotation: layer.rotation,
                                ..Default::default()
                            },
                        ))
                        .id();

                    if let Some(scene_id) = layer.scene_id {
                        id_map.entities.insert(scene_id, entity);
                    }
                    ready.push(i)
                }
            }
            RenderableHandle::Mesh(mesh_handle) => {
                match layer.mesh_type {
                    MeshType::Land => {
                        let height_mat = HeightMaterial {
                            color: LinearRgba::WHITE,
                            color_texture: Some(asset_server.load("textures/grass.png")),
                            alpha_mode: AlphaMode::Opaque,
                        };

                        let material_handle = height_materials.add(height_mat);
                        commands.spawn((
                            Mesh3d(mesh_handle.clone()), // mesh handle component
                            Transform {
                                translation: layer.position,
                                ..Default::default()
                            },
                            MeshMaterial3d(material_handle),
                        ));
                    }
                    _ => {
                        let material_handle = standard_materials.add(StandardMaterial {
                            base_color: Color::WHITE,
                            ..Default::default()
                        });
                        commands.spawn((
                            Mesh3d(mesh_handle.clone()), // mesh handle component
                            Transform {
                                translation: layer.position,
                                ..Default::default()
                            },
                            MeshMaterial3d::from(material_handle),
                        ));
                    }
                };

                ready.push(i)
            }
        }
    }
    for i in ready.iter().rev() {
        mesh_queue.items.remove(*i);
    }
}

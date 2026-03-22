use crate::textures::environment::HeightMaterial;
use benthic_default_assets::default_animations::DefaultAnimation;
use bevy::asset::RenderAssetUsages;
use bevy::mesh::skinning::SkinnedMesh;
use bevy::platform::collections::HashMap;
use bevy::prelude::*;
use bevy_gltf::{Gltf, GltfLoaderSettings};
use bevy_panorbit_camera::PanOrbitCamera;
use metaverse_messages::ui::land_update::{LandData, LandUpdate};
use metaverse_messages::ui::mesh_update::{MeshType, MeshUpdate};
use std::fs;
use std::path::PathBuf;
use uuid::Uuid;

#[derive(Resource)]
pub struct SceneIDMap {
    pub entities: HashMap<u32, Entity>,
}

#[derive(Resource)]
pub struct AgentIDMap {
    pub entities: HashMap<Uuid, AgentEntity>,
}

pub struct AgentEntity {
    pub entity: Entity,
    pub animation: PathBuf,
    pub skeleton: Entity,
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
    pub transform: Transform,
    pub parent: Option<u32>,
    pub mesh_type: MeshType,
    pub id: Option<Uuid>,
}

#[derive(Resource)]
pub struct MeshQueue {
    pub pending: Vec<Renderable>,
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

pub fn handle_mesh_update(
    mut ev_mesh_update: MessageReader<MeshUpdateEvent>,
    mut mesh_queue: ResMut<MeshQueue>,
    asset_server: Res<AssetServer>,
    mut commands: Commands,
    mut agent_id_map: ResMut<AgentIDMap>,
) {
    for renderable in ev_mesh_update.read() {
        let handle: Handle<Gltf> = asset_server.load_with_settings(
            renderable.value.path.clone(),
            |settings: &mut GltfLoaderSettings| {
                settings.load_animations = true;
            },
        );
        let transform = Transform {
            translation: renderable.value.position,
            rotation: renderable.value.rotation,
            scale: renderable.value.scale,
        };
        if renderable.value.mesh_type == MeshType::Avatar {
            let agent_root = commands.spawn((Name::new("AgentRoot"), transform)).id();

            let skeleton = commands
                .spawn((Name::new("SkeletonRoot"), AnimationPlayer::default()))
                .id();

            commands.entity(agent_root).add_child(skeleton);

            agent_id_map.entities.insert(
                renderable.value.id.unwrap(),
                AgentEntity {
                    entity: agent_root,
                    skeleton,
                    animation: DefaultAnimation::Stand.path(),
                },
            );
        };
        mesh_queue.pending.push(Renderable {
            handle: RenderableHandle::Gltf(handle),
            transform,
            parent: renderable.value.parent,
            mesh_type: renderable.value.mesh_type.clone(),
            id: renderable.value.id,
        });
    }
}

pub fn extract_gltf_meshes(
    mut commands: Commands,
    mut queue: ResMut<MeshQueue>,
    gltfs: Res<Assets<Gltf>>,
    mut scene_spawner: ResMut<SceneSpawner>,
    agents: Res<AgentIDMap>,
    mut standard_materials: ResMut<Assets<StandardMaterial>>,
    mut height_materials: ResMut<Assets<HeightMaterial>>,
    skinned_meshes: Query<(), With<SkinnedMesh>>,
    asset_server: Res<AssetServer>,
) {
    let mut ready = vec![];

    for (i, item) in queue.pending.iter().enumerate() {
        match &item.handle {
            RenderableHandle::Gltf(gltf_handle) => {
                let Some(gltf) = gltfs.get(gltf_handle) else { continue };

                // Spawn the SceneRoot with the correct transform immediately
                let scene_root = commands
                    .spawn((item.transform, Name::new("SceneRoot")))
                    .id();
                let instance = scene_spawner.spawn_as_child(gltf.scenes[0].clone(), scene_root);

                if !scene_spawner.instance_is_ready(instance) {
                    continue;
                }

                // Reparent each entity in the instance
                for entity in scene_spawner.iter_instance_entities(instance) {
                    if skinned_meshes.get(entity).is_ok() {
                        if let Some(agent) = agents.entities.get(&item.id.unwrap()) {
                            commands.entity(agent.skeleton).add_child(entity);
                        }
                    } else {
                        commands.entity(entity).add_child(scene_root);
                    }

                    // Apply original transform
                    commands.entity(entity).insert(item.transform);
                }

                // Despawn temporary root (children survive)
                scene_spawner.despawn_instance(instance);

                ready.push(i);
            }

            RenderableHandle::Mesh(mesh_handle) => {
                match item.mesh_type {
                    MeshType::Land => {
                        let height_mat = HeightMaterial {
                            color: LinearRgba::WHITE,
                            color_texture: Some(asset_server.load("textures/grass.png")),
                            alpha_mode: AlphaMode::Opaque,
                        };
                        let mat_handle = height_materials.add(height_mat);

                        commands.spawn((
                            Mesh3d(mesh_handle.clone()),
                            item.transform,
                            MeshMaterial3d(mat_handle),
                        ));
                    }

                    _ => {
                        let standard_mat = StandardMaterial {
                            base_color: Color::WHITE,
                            ..Default::default()
                        };
                        let mat_handle = standard_materials.add(standard_mat);

                        commands.spawn((
                            Mesh3d(mesh_handle.clone()),
                            item.transform,
                            MeshMaterial3d::from(mat_handle),
                        ));
                    }
                }

                ready.push(i);
            }
        }
    }

    // Remove processed items
    for i in ready.into_iter().rev() {
        queue.pending.remove(i);
    }
}

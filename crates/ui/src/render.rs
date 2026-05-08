use crate::plugin::{CameraUpdateEvent, SessionData};
use crate::textures::environment::HeightMaterial;
use benthic_protocol::default_animations::DefaultAnimation;
use benthic_protocol::messages::ui::land_update::LandUpdate;
use benthic_protocol::messages::ui::mesh_update::{MeshType, MeshUpdate};
use bevy::platform::collections::HashMap;
use bevy::prelude::*;
use bevy_gltf::{Gltf, GltfLoaderSettings};
use bevy_panorbit_camera::PanOrbitCamera;

use std::path::PathBuf;
use uuid::Uuid;

#[derive(Component)]
pub struct WaterPlane;

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

#[derive(Component, Debug)]
pub struct AgentID {
    pub id: Uuid,
}

#[derive(Component)]
pub struct MainCamera;

pub fn handle_camera_update(
    mut ev_camera_update: MessageReader<CameraUpdateEvent>,
    mut query: Query<&mut PanOrbitCamera, With<MainCamera>>,
) {
    for ev in ev_camera_update.read() {
        info!("moving the camera to {:?}", ev.value.position);
        for mut camera in &mut query {
            camera.target_focus = ev.value.position;
        }
    }
}

pub fn follow_gltf_with_offset(
    gltf_models: Query<(&Transform, &AgentID), Without<PanOrbitCamera>>,
    session_data: ResMut<SessionData>,
    mut cameras: Query<&mut PanOrbitCamera, With<MainCamera>>,
) {
    if let Some(login_response) = &session_data.login_response {
        if let Ok(mut camera) = cameras.single_mut() {
            for (model_transform, agent_id) in gltf_models.iter() {
                if agent_id.id == login_response.agent_id {
                    camera.target_focus = model_transform.translation;
                    break;
                }
            }
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

            agent_id_map.entities.insert(
                renderable.value.id.unwrap(),
                AgentEntity {
                    entity: agent_root,
                    skeleton: agent_root,
                    animation: DefaultAnimation::Stand.path(),
                },
            );
        }

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
    mut standard_materials: ResMut<Assets<StandardMaterial>>,
    mut height_materials: ResMut<Assets<HeightMaterial>>,
    asset_server: Res<AssetServer>,
) {
    let mut ready = vec![];

    for (i, item) in queue.pending.iter().enumerate() {
        match &item.handle {
            RenderableHandle::Gltf(gltf_handle) => {
                let Some(gltf) = gltfs.get(gltf_handle) else { continue };
                let scene_root = if let Some(agent_id) = item.id {
                    commands
                        .spawn((
                            item.transform,
                            Name::new("SceneRoot"),
                            AgentID { id: agent_id },
                        ))
                        .id()
                } else {
                    commands
                        .spawn((item.transform, Name::new("SceneRoot")))
                        .id()
                };
                let instance = scene_spawner.spawn_as_child(gltf.scenes[0].clone(), scene_root);

                for entity in scene_spawner.iter_instance_entities(instance) {
                    commands.entity(entity).insert(item.transform);
                }

                ready.push(i);
            }

            RenderableHandle::Mesh(mesh_handle) => {
                match item.mesh_type {
                    MeshType::Land => {
                        //TODO: this height material is unfinished. the shader attached to this
                        //does not have any ability to shade.
                        //
                        //let height_mat = HeightMaterial {
                        //    color: LinearRgba::WHITE,
                        //    color_texture: Some(asset_server.load("textures/grass.png")),
                        //    alpha_mode: AlphaMode::Opaque,
                        //};
                        //let mat_handle = height_materials.add(height_mat);

                        let standard_mat = StandardMaterial {
                            base_color: Color::WHITE,
                            ..Default::default()
                        };
                        let mat_handle = standard_materials.add(standard_mat);

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

    for i in ready.into_iter().rev() {
        queue.pending.remove(i);
    }
}

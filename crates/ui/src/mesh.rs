use crate::core_plugin::{CameraUpdateEvent, MeshUpdateEvent};
use crate::plugin::SessionData;
use benthic_protocol::messages::ui::mesh_update::MeshType;
use bevy::animation::AnimatedBy;
use bevy::camera::visibility::DynamicSkinnedMeshBounds;
use bevy::mesh::skinning::SkinnedMesh;
use bevy::platform::collections::HashMap;
use bevy::prelude::*;
use bevy_gltf::{Gltf, GltfAssetLabel, GltfMaterialName, GltfMeshName, GltfSceneName};
use bevy_panorbit_camera::PanOrbitCamera;
use bevy_world_serialization::WorldAssetRoot;

use std::path::PathBuf;
use uuid::Uuid;

pub struct BenthicMeshPlugin;
impl Plugin for BenthicMeshPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(AgentIDMap {
            entities: HashMap::new(),
        })
        .insert_resource(MeshQueue { pending: vec![] })
        .insert_resource(SceneIDMap {
            entities: HashMap::new(),
        })
        .register_type::<GltfSceneName>()
        .register_type::<Transform>()
        .register_type::<GlobalTransform>()
        .register_type::<TransformTreeChanged>()
        .register_type::<Children>()
        .register_type::<Visibility>()
        .register_type::<ChildOf>()
        .register_type::<InheritedVisibility>()
        .register_type::<ViewVisibility>()
        .register_type::<AnimationPlayer>()
        .register_type::<Name>()
        .register_type::<Mesh3d>()
        .register_type::<bevy::camera::primitives::Aabb>()
        .register_type::<SkinnedMesh>()
        .register_type::<GltfMeshName>()
        .register_type::<GltfMaterialName>()
        .register_type::<AnimatedBy>()
        .register_type::<DynamicSkinnedMeshBounds>()
        .add_systems(Update, handle_camera_update)
        .add_systems(Update, follow_gltf_with_offset)
        .add_systems(Update, render_land)
        .add_systems(Update, render_meshes)
        .add_systems(Update, handle_mesh_update);
    }
}

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
    pub animation: Option<PathBuf>,
    pub skeleton: Entity,
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
    if let Some(login_response) = &session_data.login_response
        && let Ok(mut camera) = cameras.single_mut()
    {
        for (model_transform, agent_id) in gltf_models.iter() {
            if agent_id.id == login_response.agent_id {
                camera.target_focus = model_transform.translation;
                break;
            }
        }
    }
}

pub fn handle_mesh_update(
    mut ev_mesh_update: MessageReader<MeshUpdateEvent>,
    asset_server: Res<AssetServer>,
    mut commands: Commands,
    mut agent_id_map: ResMut<AgentIDMap>,
) {
    for renderable in ev_mesh_update.read() {
        let transform = Transform {
            translation: renderable.value.position,
            rotation: renderable.value.rotation,
            scale: renderable.value.scale,
        };

        let scene =
            asset_server.load(GltfAssetLabel::Scene(0).from_asset(renderable.value.path.clone()));

        let mut entity_commands = commands.spawn((
            WorldAssetRoot(scene),
            transform,
            Visibility::Visible,
            Name::new("SceneRoot"),
        ));

        if renderable.value.mesh_type == MeshType::Avatar {
            let agent_id = renderable.value.id.unwrap();

            entity_commands.insert(AgentID { id: agent_id });

            let entity = entity_commands.id();

            agent_id_map.entities.insert(
                agent_id,
                AgentEntity {
                    entity,
                    skeleton: entity,
                    animation: None,
                },
            );

            info!("Spawned avatar {:?} as {:?}", agent_id, entity);
        }
    }
}

pub fn render_land(
    mut commands: Commands,
    mut queue: ResMut<MeshQueue>,
    mut standard_materials: ResMut<Assets<StandardMaterial>>,
) {
    let mut ready = vec![];

    for (i, item) in queue.pending.iter().enumerate() {
        let RenderableHandle::Mesh(mesh_handle) = &item.handle else {
            continue;
        };

        if item.mesh_type != MeshType::Land {
            continue;
        }

        let standard_mat = standard_materials.add(StandardMaterial {
            base_color: Color::WHITE,
            ..default()
        });

        commands.spawn((
            Mesh3d(mesh_handle.clone()),
            item.transform,
            MeshMaterial3d(standard_mat),
        ));

        ready.push(i);
    }

    for i in ready.into_iter().rev() {
        queue.pending.remove(i);
    }
}

pub fn render_meshes(
    mut commands: Commands,
    mut queue: ResMut<MeshQueue>,
    mut standard_materials: ResMut<Assets<StandardMaterial>>,
) {
    let mut ready = vec![];

    for (i, item) in queue.pending.iter().enumerate() {
        let RenderableHandle::Mesh(mesh_handle) = &item.handle else {
            continue;
        };

        if item.mesh_type == MeshType::Land {
            continue;
        }

        let mat_handle = standard_materials.add(StandardMaterial {
            base_color: Color::WHITE,
            ..default()
        });

        commands.spawn((
            Mesh3d(mesh_handle.clone()),
            item.transform,
            MeshMaterial3d(mat_handle),
        ));

        ready.push(i);
    }

    for i in ready.into_iter().rev() {
        queue.pending.remove(i);
    }
}

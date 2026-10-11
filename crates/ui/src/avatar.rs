use crate::core_plugin::{AvatarUpdateEvent, CameraUpdateEvent};
use crate::mesh::BenthicMeshPlugin;
use crate::plugin::{MainCamera, SessionData};
use bevy::app::App;
use bevy::platform::collections::HashMap;
use bevy::prelude::*;
use bevy_panorbit_camera::PanOrbitCamera;
use std::path::PathBuf;
use uuid::Uuid;

#[derive(Resource)]
pub struct AgentIDMap {
    pub entities: HashMap<Uuid, AgentEntity>,
}
#[derive(Component, Debug)]
pub struct AgentID {
    pub id: Uuid,
}

pub struct AgentEntity {
    pub entity: Entity,
    pub animation: Option<PathBuf>,
    pub skeleton: Entity,
}

pub struct AvatarPlugin;
impl Plugin for AvatarPlugin {
    fn build(&self, app: &mut App) {
        if !app.is_plugin_added::<BenthicMeshPlugin>() {
            app.add_plugins(BenthicMeshPlugin);
        }
        app.insert_resource(AgentIDMap {
            entities: HashMap::new(),
        })
        .add_systems(Update, handle_avatar_update)
        .add_systems(Update, handle_camera_update)
        .add_systems(Update, follow_gltf_with_offset);
    }
}

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

pub fn handle_avatar_update(
    mut events: MessageReader<AvatarUpdateEvent>,
    asset_server: Res<AssetServer>,
    mut commands: Commands,
    mut agent_id_map: ResMut<AgentIDMap>,
) {
    for event in events.read() {
        let avatar = &event.value;
        let Some(agent_id) = avatar.id else {
            warn!("Avatar update missing agent ID");
            continue;
        };

        let scene = asset_server.load(GltfAssetLabel::Scene(0).from_asset(avatar.path.clone()));

        let mut entity_commands = commands.spawn((
            Transform {
                translation: avatar.position,
                rotation: avatar.rotation,
                scale: Vec3::ONE,
            },
            Visibility::Visible,
            Name::new("Avatar"),
            AgentID { id: agent_id },
        ));

        entity_commands.with_child((
            WorldAssetRoot(scene),
            Transform::default(),
            Visibility::Visible,
            Name::new("AvatarMesh"),
        ));

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

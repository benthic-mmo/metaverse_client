use std::collections::VecDeque;

use crate::mesh::BenthicMeshPlugin;
use crate::plugin::Sockets;
use crate::{core_plugin::MeshUpdateEvent, plugin::send_packet_to_core};
use benthic_protocol::messages::{
    simulator::orphan_objects::OrphanedObject, ui_protocol::UIResponse,
};
use bevy::asset::LoadState;
use bevy::diagnostic::DiagnosticsStore;
use bevy::platform::collections::HashMap;
use bevy::prelude::*;
use bevy_gltf::GltfAssetLabel;
use bevy_world_serialization::WorldAssetRoot;
use uuid::Uuid;

pub struct BenthicObjectPlugin;

#[derive(Component)]
pub struct ObjectData {
    pub id: Uuid,
    pub scene_id: Option<u32>,
    pub scale: Vec3,
    pub position: Vec3,
    pub rotation: Quat,
    pub parent: Option<u32>,
}

const MAX_IN_FLIGHT: usize = 3;

use std::time::{Duration, Instant};

pub struct PendingChild {
    pub entity: Entity,
    pub scene_id: u32,
    pub since: Instant,
    pub last_reported: Option<Instant>,
}

#[derive(Resource)]
pub struct SceneIDMap {
    pub entities: HashMap<u32, Entity>,
    pub pending_children: HashMap<u32, Vec<PendingChild>>,
    pub spawned: HashMap<Uuid, Entity>,
    pub in_flight: Vec<(Handle<WorldAsset>, Entity)>,
    pub load_queue: VecDeque<QueuedLoad>,
}

pub struct QueuedLoad {
    path: String,
    mesh_child: Entity,
}

impl Plugin for BenthicObjectPlugin {
    fn build(&self, app: &mut App) {
        if !app.is_plugin_added::<BenthicMeshPlugin>() {
            app.add_plugins(BenthicMeshPlugin);
        }
        app.insert_resource(SceneIDMap {
            entities: HashMap::new(),
            pending_children: HashMap::new(),
            spawned: HashMap::new(),
            in_flight: Vec::new(),
            load_queue: VecDeque::new(),
        })
        .add_systems(
            Update,
            (handle_mesh_update, drain_load_queue, check_orphaned_objects),
        );
    }
}

pub fn handle_mesh_update(
    mut ev_mesh_update: MessageReader<MeshUpdateEvent>,
    mut commands: Commands,
    mut scene_id_map: ResMut<SceneIDMap>,
) {
    let mut received = 0;

    for renderable in ev_mesh_update.read() {
        received += 1;
        let object = &renderable.value;
        let object_id = object.full_id.unwrap_or(Uuid::nil());

        let scale = if object.scale.abs_diff_eq(Vec3::ZERO, f32::EPSILON) {
            Vec3::ONE
        } else {
            object.scale
        };

        // Spawn a new mesh child for every object instance.
        let mesh_child = commands
            .spawn((
                Transform::from_scale(scale),
                Visibility::Visible,
                Name::new("Mesh"),
            ))
            .id();

        let mut entity_commands = commands.spawn((
            Transform {
                translation: object.position,
                rotation: object.rotation,
                scale: Vec3::ONE,
            },
            Visibility::Visible,
            Name::new("SceneRoot"),
            Pickable::default(),
            ObjectData {
                id: object_id,
                scene_id: object.scene_id,
                scale,
                position: object.position,
                rotation: object.rotation,
                parent: object.parent,
            },
        ));

        entity_commands.add_child(mesh_child);
        let entity = entity_commands.id();

        // Queue the mesh load; multiple instances can share the same path.
        scene_id_map.load_queue.push_back(QueuedLoad {
            path: object.path.to_string_lossy().into_owned(),
            mesh_child,
        });

        let Some(scene_id) = object.scene_id else {
            continue;
        };

        scene_id_map.entities.insert(scene_id, entity);

        if let Some(parent_id) = object.parent
            && parent_id != 0
        {
            if let Some(&parent_entity) = scene_id_map.entities.get(&parent_id) {
                commands.entity(entity).insert(ChildOf(parent_entity));
            } else {
                scene_id_map
                    .pending_children
                    .entry(parent_id)
                    .or_default()
                    .push(PendingChild {
                        entity,
                        scene_id,
                        since: Instant::now(),
                        last_reported: None,
                    });
            }
        }

        if let Some(children) = scene_id_map.pending_children.remove(&scene_id) {
            for child in children {
                commands.entity(child.entity).insert(ChildOf(entity));
            }
        }
    }

    if received > 0 {
        info!("Mesh updates: {received} received");
    }
}

pub fn check_orphaned_objects(mut scene_id_map: ResMut<SceneIDMap>, sockets: Res<Sockets>) {
    let timeout = Duration::from_secs(20 * 60);
    let report_interval = Duration::from_secs(10);

    for (parent_id, children) in &mut scene_id_map.pending_children {
        children.retain_mut(|child| {
            let elapsed = child.since.elapsed();

            // Permanently remove objects pending for 20 minutes.
            if elapsed >= timeout {
                warn!(
                    "Removing expired orphan: scene ID {}, missing parent {}",
                    child.scene_id, parent_id
                );
                return false;
            }

            // Report after 10 seconds, then every 10 seconds.
            if elapsed >= report_interval
                && child
                    .last_reported
                    .is_none_or(|last| last.elapsed() >= report_interval)
            {
                let packet = UIResponse::new_orphaned_object(OrphanedObject {
                    local_id: child.scene_id,
                    parent_id: *parent_id,
                })
                .to_bytes();

                match send_packet_to_core(&packet, &sockets) {
                    Ok(()) => {
                        child.last_reported = Some(Instant::now());
                    }
                    Err(err) => {
                        error!("Failed to send OrphanedObject response: {err}");
                    }
                }
            }

            true
        });
    }

    scene_id_map
        .pending_children
        .retain(|_, children| !children.is_empty());
}

// this needs to be done this way because of a bug that deadlocks bevy if too many multi-texture
// gltfs spawn at the same time.
fn drain_load_queue(
    asset_server: Res<AssetServer>,
    mut scene_id_map: ResMut<SceneIDMap>,
    mut commands: Commands,
) {
    // Drop finished/failed loads from the in-flight tracker.
    scene_id_map.in_flight.retain(|(h, _)| {
        matches!(
            asset_server.get_load_state(h.id()),
            None | Some(LoadState::Loading) | Some(LoadState::NotLoaded)
        )
    });

    while scene_id_map.in_flight.len() < MAX_IN_FLIGHT {
        let Some(req) = scene_id_map.load_queue.pop_front() else {
            break;
        };

        let handle: Handle<WorldAsset> =
            asset_server.load(GltfAssetLabel::Scene(0).from_asset(req.path.clone()));

        commands
            .entity(req.mesh_child)
            .insert(WorldAssetRoot(handle.clone()));
        scene_id_map.in_flight.push((handle, req.mesh_child));
    }
}

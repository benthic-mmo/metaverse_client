use std::collections::VecDeque;

use crate::core_plugin::MeshUpdateEvent;
use crate::mesh::BenthicMeshPlugin;
use bevy::asset::{AssetLoadFailedEvent, LoadState, UntypedAssetLoadFailedEvent};
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

const MAX_IN_FLIGHT: usize = 4;

#[derive(Resource)]
pub struct SceneIDMap {
    pub entities: HashMap<u32, Entity>,
    pub pending_children: HashMap<u32, Vec<Entity>>,
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
        .add_systems(Update, (handle_mesh_update, drain_load_queue));
    }
}

pub fn handle_mesh_update(
    mut ev_mesh_update: MessageReader<MeshUpdateEvent>,
    mut commands: Commands,
    mut scene_id_map: ResMut<SceneIDMap>,
) {
    let mut received = 0;
    let mut skipped = 0;

    for renderable in ev_mesh_update.read() {
        received += 1;
        let object = &renderable.value;
        let object_id = object.id.unwrap_or(Uuid::nil());

        if let Some(&existing) = scene_id_map.spawned.get(&object_id) {
            commands.entity(existing).insert(Transform {
                translation: object.position,
                rotation: object.rotation,
                scale: Vec3::ONE,
            });
            skipped += 1;
            continue;
        }

        let scale = if object.scale.abs_diff_eq(Vec3::ZERO, f32::EPSILON) {
            Vec3::ONE
        } else {
            object.scale
        };

        // Spawn the mesh child WITHOUT a handle yet.
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
        scene_id_map.spawned.insert(object_id, entity);

        // Queue the load instead of starting it.
        scene_id_map.load_queue.push_back(QueuedLoad {
            path: object.path.to_string_lossy().into_owned(),
            mesh_child,
        });

        let entity = entity_commands.id();
        scene_id_map.spawned.insert(object_id, entity);

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
                    .push(entity);
            }
        }

        if let Some(children) = scene_id_map.pending_children.remove(&scene_id) {
            for child in children {
                commands.entity(child).insert(ChildOf(entity));
            }
        }
    }

    if received > 0 {
        info!("Mesh updates: {received} received, {skipped} skipped as duplicates");
    }
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

// used to debug throughput issues
fn io_throughput(
    asset_server: Res<AssetServer>,
    scene_id_map: Res<SceneIDMap>,
    diagnostics: Res<DiagnosticsStore>,
) {
    let started = diagnostics
        .get(&AssetServer::STARTED_LOAD_COUNT)
        .and_then(|d| d.smoothed())
        .unwrap_or(0.0);

    let mut loading = 0;
    let mut loaded = 0;
    let mut failed = 0;
    for (handle, _) in &scene_id_map.in_flight {
        match asset_server.get_load_state(handle.id()) {
            Some(LoadState::Loading) | Some(LoadState::NotLoaded) => loading += 1,
            Some(LoadState::Loaded) => loaded += 1,
            Some(LoadState::Failed(_)) => failed += 1,
            None => {}
        }
    }
    let queued = scene_id_map.load_queue.len();

    info!(
        "started={started} in_flight={} queued={queued} loading={loading} loaded={loaded} failed={failed}",
        scene_id_map.in_flight.len()
    );
}

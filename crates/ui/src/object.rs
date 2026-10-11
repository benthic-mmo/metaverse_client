use crate::core_plugin::MeshUpdateEvent;
use crate::mesh::BenthicMeshPlugin;
use bevy::platform::collections::HashMap;
use bevy::prelude::*;
use bevy_gltf::GltfAssetLabel;
use bevy_world_serialization::WorldAssetRoot;
use uuid::Uuid;

pub struct BenthicObjectPlugin;

impl Plugin for BenthicObjectPlugin {
    fn build(&self, app: &mut App) {
        if !app.is_plugin_added::<BenthicMeshPlugin>() {
            app.add_plugins(BenthicMeshPlugin);
        }
        app.insert_resource(SceneIDMap {
            entities: HashMap::new(),
        })
        .add_systems(Update, handle_mesh_update);
    }
}

#[derive(Resource)]
pub struct SceneIDMap {
    pub entities: HashMap<u32, Entity>,
}

#[derive(Component)]
pub struct ObjectData {
    pub id: Uuid,
    pub scene_id: Option<u32>,
    pub scale: Vec3,
    pub position: Vec3,
    pub rotation: Quat,
    pub parent: Option<u32>,
}

pub fn handle_mesh_update(
    mut ev_mesh_update: MessageReader<MeshUpdateEvent>,
    asset_server: Res<AssetServer>,
    mut commands: Commands,
    mut scene_id_map: ResMut<SceneIDMap>,
) {
    for renderable in ev_mesh_update.read() {
        let object = &renderable.value;

        let scene = asset_server.load(GltfAssetLabel::Scene(0).from_asset(object.path.clone()));

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
                id: object.id.unwrap_or(Uuid::nil()),
                scene_id: object.scene_id,
                scale: object.scale,
                position: object.position,
                rotation: object.rotation,
                parent: object.parent,
            },
        ));

        entity_commands.with_child((
            WorldAssetRoot(scene),
            Transform::from_scale(object.scale),
            Visibility::Visible,
            Name::new("Mesh"),
        ));

        let entity = entity_commands.id();

        if let Some(scene_id) = object.scene_id {
            scene_id_map.entities.insert(scene_id, entity);

            if let Some(parent_id) = object.parent
                && parent_id != 0
                && let Some(&parent_entity) = scene_id_map.entities.get(&parent_id)
            {
                commands.entity(entity).insert(ChildOf(parent_entity));
            }
        }
    }
}

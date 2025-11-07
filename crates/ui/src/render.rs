use bevy::prelude::*;
use bevy_panorbit_camera::PanOrbitCamera;
use metaverse_messages::ui::mesh_update::{MeshType, MeshUpdate};

#[derive(Message)]
pub struct MeshUpdateEvent {
    pub value: MeshUpdate,
}

pub enum RenderableHandle {
    Gltf(Handle<Gltf>),
    Mesh(Handle<Mesh>),
}

#[derive(Resource)]
pub struct Renderable {
    pub handle: RenderableHandle,
    pub position: Vec3,
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
        Transform::from_xyz(200.0, 100.0, 200.0),
    ));
    commands.spawn((
        Transform {
            translation: Vec3::new(678.0, 471.0, 962.0),
            ..default()
        },
        PanOrbitCamera::default(),
    ));
}

pub fn handle_mesh_update(
    mut ev_mesh_update: MessageReader<MeshUpdateEvent>,
    mut mesh_queue: ResMut<MeshQueue>,
    asset_server: Res<AssetServer>,
) {
    for renderable in ev_mesh_update.read() {
        // this needs to be done because Bevy and the Core crate might be using different versions of Glam
        let position = Vec3::from_array([
            renderable.value.position.x,
            renderable.value.position.z,
            renderable.value.position.y,
        ]);

        let handle: Handle<Gltf> = asset_server.load(renderable.value.path.clone());
        mesh_queue.items.push(Renderable {
            handle: RenderableHandle::Gltf(handle),
            position,
            mesh_type: renderable.value.mesh_type.clone(),
        });
    }
}

pub fn check_model_loaded(
    mut commands: Commands,
    mut mesh_queue: ResMut<MeshQueue>,
    layer_assets: Res<Assets<Gltf>>,
    meshes: Res<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let mut ready = vec![];
    for (i, layer) in mesh_queue.items.iter().enumerate() {
        let white_material = materials.add(StandardMaterial {
            base_color: Color::WHITE,
            ..Default::default()
        });

        match &layer.handle {
            RenderableHandle::Gltf(gltf) => {
                if let Some(gltf) = layer_assets.get(gltf) {
                    commands.spawn((
                        SceneRoot(gltf.scenes[0].clone()),
                        Transform {
                            translation: layer.position,
                            scale: match layer.mesh_type {
                                MeshType::Avatar => Vec3::splat(20.0),
                                MeshType::Land => Vec3::ONE,
                            },
                            ..Default::default()
                        },
                        MeshMaterial3d::from(white_material.clone()),
                    ));
                    ready.push(i);
                }
            }
            RenderableHandle::Mesh(mesh) => {
                if meshes.get(mesh).is_some() {
                    commands.spawn((
                        Mesh3d(mesh.clone()),
                        Transform {
                            translation: layer.position,
                            ..Default::default()
                        },
                        MeshMaterial3d::from(white_material.clone()),
                    ));
                    ready.push(i);
                }
            }
        }
    }

    for i in ready.iter().rev() {
        mesh_queue.items.remove(*i);
    }
}

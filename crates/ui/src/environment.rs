use benthic_protocol::messages::ui::land_update::{LandData, LandUpdate};
use benthic_protocol::messages::ui::mesh_update::MeshType;
use bevy::asset::RenderAssetUsages;
use bevy::prelude::*;
use std::fs;

use crate::render::{MeshQueue, Renderable, RenderableHandle};
use crate::textures::environment::HeightMaterial;

#[derive(Message)]
pub struct LandUpdateEvent {
    pub value: LandUpdate,
}

pub struct LandPlugin;

impl Plugin for LandPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<LandUpdateEvent>()
            .add_systems(Update, handle_land_update)
            .add_plugins(MaterialPlugin::<HeightMaterial>::default());
    }
}

fn handle_land_update(
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

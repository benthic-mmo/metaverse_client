use crate::core_plugin::LandUpdateEvent;
use crate::textures::environment::HeightMaterial;
use benthic_protocol::messages::ui::land_update::LandData;
use bevy::asset::RenderAssetUsages;
use bevy::prelude::*;
use std::fs;

pub struct LandPlugin;

impl Plugin for LandPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, handle_land_update)
            .add_plugins(MaterialPlugin::<HeightMaterial>::default());
    }
}

fn handle_land_update(
    mut commands: Commands,
    mut ev_land_update: MessageReader<LandUpdateEvent>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    for patch in ev_land_update.read() {
        let land = &patch.value;

        let Ok(json_str) = fs::read_to_string(&land.path) else {
            error!("Failed to read file {}", land.path.to_string_lossy());
            continue;
        };

        let Ok(land_data) = serde_json::from_str::<LandData>(&json_str) else {
            error!(
                "Failed to deserialize JSON: {}",
                land.path.to_string_lossy()
            );
            continue;
        };

        let mut mesh = Mesh::new(
            bevy::mesh::PrimitiveTopology::TriangleList,
            RenderAssetUsages::RENDER_WORLD,
        );

        mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, land_data.vertices);
        mesh.insert_indices(bevy::mesh::Indices::U16(land_data.indices));
        mesh.compute_smooth_normals();

        let mesh_handle = meshes.add(mesh);

        let material = materials.add(StandardMaterial {
            base_color: Color::WHITE,
            ..default()
        });

        let conversion = Quat::from_rotation_z(std::f32::consts::PI)
            * Quat::from_rotation_x(std::f32::consts::FRAC_PI_2);

        commands.spawn((
            Mesh3d(mesh_handle),
            MeshMaterial3d(material),
            Transform {
                translation: conversion.mul_vec3(land_data.position),
                rotation: conversion,
                ..default()
            },
        ));
    }
}

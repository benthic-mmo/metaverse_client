use std::{f32::consts::TAU, path::PathBuf};

use benthic_protocol::{
    objects::{GeneratorObject, ParametricPrimData},
    render_data::RenderObject,
    session::{cache_enabled, create_sub_object_dir},
};
use glam::Vec3;
use metaverse_mesh::mesh::generate::generate_object_mesh;
use metaverse_messages::utils::{
    path::{HollowShape, PathCurve, PrimPath, ProfileShape},
    texture_entry::TextureEntry,
};
use metaverse_store::initialize_sqlite::Cache;
use uuid::Uuid;

use crate::{
    errors::ObjectUpdateError,
    object_updates::{ObjectUpdateAction, RenderObjectData},
};

/// This contains one layer of vertices.
struct MeshLayer(Vec<Vec3>);

pub async fn handle_parametric_prim(
    cache: &Cache,
    out_dir: &PathBuf,
    object: &ParametricPrimData<TextureEntry, PrimPath>,
) -> Result<Vec<ObjectUpdateAction>, ObjectUpdateError> {
    let mut actions: Vec<ObjectUpdateAction> = Vec::new();
    let profile_layer = build_profile(
        object.path_data.hollow_shape,
        object.path_data.profile_hollow,
        object.path_data.profile_shape,
        object.path_data.scale_x,
        object.path_data.scale_y,
    )?;

    let mesh_layers = follow_path(object.path_data.clone(), &profile_layer)?;
    let (vertices, indices) = stitch_faces(mesh_layers)?;
    let normals = generate_normals(&vertices, &indices);

    let path_hash = object.path_data.hash();

    let base_dir = create_sub_object_dir(out_dir, &format!("{path_hash:016x}"))?;

    let file_path = base_dir.join(format!("{}.json", path_hash));
    let glb_path = base_dir.join(format!("{}.glb", path_hash));

    if !glb_path.exists() || !cache_enabled() {
        if !file_path.exists() || !cache_enabled() {
            let render_object = RenderObject {
                name: "Prim".to_string(),
                id: object.full_id,
                vertices,
                indices,
                skin: None,
                texture: None,
                uv: None,
                normals: Some(normals),
            };

            let json = serde_json::to_vec_pretty(&render_object)?;
            std::fs::write(&file_path, json)?;
        }

        generate_object_mesh(file_path.clone(), glb_path.clone())?;
    }

    actions.push(ObjectUpdateAction::Render(RenderObjectData {
        mesh_path: Some(glb_path),
        asset_id: Uuid::nil(),
        base_dir,
        object: GeneratorObject {
            full_id: object.full_id,
            local_id: object.local_id,
            parent_id: object.parent,
            position: object.position,
            scale: object.scale,
            rotation: object.rotation,
        },
        download: None,
        retry_count: 0,
    }));

    Ok(actions)
}

fn stitch_faces(layers: Vec<MeshLayer>) -> Result<(Vec<Vec3>, Vec<u16>), ObjectUpdateError> {
    let mut vertices = Vec::new();
    let mut indices = Vec::new();

    if layers.len() < 2 {
        return Ok((vertices, indices));
    }

    let layer_size = layers[0].0.len();

    if layer_size < 3 {
        return Ok((vertices, indices));
    }

    // Flatten all layers into one vertex array.
    for layer in &layers {
        if layer.0.len() != layer_size {
            return Err(ObjectUpdateError::Unimplemented {
                feature: "Mesh layers have mismatched vertex counts".to_string(),
            });
        }

        vertices.extend_from_slice(&layer.0);
    }

    // Stitch every layer to the next layer.
    for layer_index in 0..layers.len() - 1 {
        let current_start = layer_index * layer_size;
        let next_start = (layer_index + 1) * layer_size;

        for i in 0..layer_size {
            let next_i = (i + 1) % layer_size;

            let a = (current_start + i) as u16;
            let b = (current_start + next_i) as u16;
            let c = (next_start + i) as u16;
            let d = (next_start + next_i) as u16;

            // Two triangles forming the quad.
            indices.extend_from_slice(&[a, c, b, b, c, d]);
        }
    }

    Ok((vertices, indices))
}
fn generate_normals(vertices: &[Vec3], indices: &[u16]) -> Vec<Vec3> {
    let mut normals = vec![Vec3::ZERO; vertices.len()];

    for triangle in indices.chunks_exact(3) {
        let a = triangle[0] as usize;
        let b = triangle[1] as usize;
        let c = triangle[2] as usize;

        let edge1 = vertices[b] - vertices[a];
        let edge2 = vertices[c] - vertices[a];
        let normal = edge1.cross(edge2);

        normals[a] += normal;
        normals[b] += normal;
        normals[c] += normal;
    }

    for normal in &mut normals {
        *normal = normal.try_normalize().unwrap_or(Vec3::Z);
    }

    normals
}
fn follow_path(
    path_data: PrimPath,
    face_vertices: &MeshLayer,
) -> Result<Vec<MeshLayer>, ObjectUpdateError> {
    let mut layers = Vec::new();
    match path_data.curve {
        PathCurve::Circle => {
            let y_path_scale = path_data.scale_y * 0.5;
            let ring_radius = 0.5 - y_path_scale;
            let start_angle =
                TAU * path_data.begin * path_data.revolutions - path_data.shear_y * 0.9;
            let end_angle = TAU * path_data.end * path_data.revolutions - path_data.shear_y * 0.9;
            let step = TAU / path_data.curve.high_resolution_steps();

            let mut angle = start_angle;

            loop {
                let node_pos = Vec3::new(0.0, angle.cos() * ring_radius, angle.sin() * ring_radius);

                let mut vertices = Vec::new();
                // Move every profile vertex to this position on the path.
                for vertex in face_vertices.0.clone() {
                    vertices.push(vertex + node_pos);
                }
                layers.push(MeshLayer(vertices));

                if angle >= end_angle {
                    break;
                }

                angle = (angle + step).min(end_angle);
            }

            Ok(layers)
        }
        _ => Err(ObjectUpdateError::Unimplemented {
            feature: "non-circle paths".to_string(),
        })?,
    }
}

fn build_profile(
    hollow_shape: HollowShape,
    hollow_size: f32,
    profile: ProfileShape,
    x_scale: f32,
    y_scale: f32,
) -> Result<MeshLayer, ObjectUpdateError> {
    let sides = profile.default_sides_high();
    let mut vertices = Vec::new();
    match profile {
        ProfileShape::Circle => {
            for _ in 0..sides {
                let profile_vertices = build_profile_vertices(profile, x_scale, y_scale)?;
                vertices.extend(profile_vertices.clone());
                // if hollow_size is nothing, don't handle  hollow size
                if hollow_size <= 0.0 {
                    continue;
                }
                match hollow_shape {
                    HollowShape::Circle | HollowShape::Same => {
                        for vertex in profile_vertices {
                            vertices.push(vertex * hollow_size)
                        }
                    }
                    HollowShape::Square => {
                        let hollow_vertices =
                            build_profile_vertices(ProfileShape::Square, x_scale, y_scale)?;
                        for vertex in hollow_vertices {
                            vertices.push(vertex * hollow_size)
                        }
                    }

                    HollowShape::Triangle => {
                        let hollow_vertices =
                            build_profile_vertices(ProfileShape::EqualTriangle, x_scale, y_scale)?;
                        for vertex in hollow_vertices {
                            vertices.push(vertex * hollow_size)
                        }
                    }
                    HollowShape::Unknown => {
                        let hollow_vertices =
                            build_profile_vertices(ProfileShape::Unknown, x_scale, y_scale)?;
                        for vertex in hollow_vertices {
                            vertices.push(vertex * hollow_size)
                        }
                    }
                };
            }
        }
        _ => Err(ObjectUpdateError::Unimplemented {
            feature: "Non-circle profile".to_string(),
        })?,
    };

    Ok(MeshLayer(vertices))
}

fn build_profile_vertices(
    profile: ProfileShape,
    x_scale: f32,
    y_scale: f32,
) -> Result<Vec<Vec3>, ObjectUpdateError> {
    let sides = profile.default_sides_high();
    let mut vertices = Vec::with_capacity(sides);

    match profile {
        ProfileShape::Circle => {
            for i in 0..sides {
                let angle = 2.0 * std::f32::consts::PI * i as f32 / sides as f32;

                vertices.push(Vec3::new(angle.cos() * x_scale, angle.sin() * y_scale, 0.0));
            }
        }
        _ => {
            return Err(ObjectUpdateError::Unimplemented {
                feature: format!("Profile shape {profile:?}"),
            });
        }
    }

    Ok(vertices)
}

use std::{
    collections::HashMap,
    f32::consts::TAU,
    path::{Path, PathBuf},
};

use benthic_protocol::{
    objects::{GeneratorObject, ParametricPrimData},
    render_data::{RenderFace, RenderObject},
    session::{cache_enabled, create_sub_object_dir},
};
use glam::Vec3;
use metaverse_mesh::mesh::generate::generate_object_mesh;
use metaverse_messages::utils::{
    path::{HollowShape, PathCurve, PrimPath, ProfileShape},
    texture_entry::TextureEntries,
};
use metaverse_store::initialize_sqlite::Cache;
use uuid::Uuid;

use crate::{
    errors::ObjectUpdateError,
    object_handler::handle_texture,
    object_updates::{ObjectUpdateAction, RenderObjectData},
};

/// This contains one layer of vertices.
struct MeshLayer(Vec<Vec3>);

pub async fn handle_parametric_prim(
    out_dir: &Path,
    object: &ParametricPrimData<TextureEntries, PrimPath>,
    server_endpoint: String,
) -> Result<Vec<ObjectUpdateAction>, ObjectUpdateError> {
    let mut actions: Vec<ObjectUpdateAction> = Vec::new();

    let mut texture_paths = HashMap::new();
    if let Some(texture) = &object.texture {
        let texture_path = handle_texture(
            out_dir.to_path_buf(),
            texture.default.texture_id,
            server_endpoint.clone(),
        )
        .await?;
        texture_paths.insert(u32::MAX, texture_path);
        for (face_index, face) in &texture.faces {
            let texture_path = handle_texture(
                out_dir.to_path_buf(),
                face.texture_id,
                server_endpoint.clone(),
            )
            .await?;

            texture_paths.insert(*face_index, texture_path);
        }
    }

    let profile_layer = build_profile(
        object.path_data.hollow_shape,
        object.path_data.profile_hollow,
        object.path_data.profile_shape,
        object.path_data.scale_x,
        object.path_data.scale_y,
    )?;

    let mesh_layers = follow_path(object.path_data.clone(), &profile_layer)?;
    let has_hollow = object.path_data.profile_hollow > 0.0;
    let (vertices, indices) = stitch_faces(mesh_layers, has_hollow)?;
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
                faces: vec![RenderFace {
                    face_index: 0,
                    vertices,
                    indices,
                    uv: Vec::new(),
                    normals: Some(normals),
                    texture: None,
                    weights: None,
                }],
                skin: None,
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

fn stitch_faces(
    layers: Vec<MeshLayer>,
    has_hollow: bool,
) -> Result<(Vec<Vec3>, Vec<u32>), ObjectUpdateError> {
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

            let a = (current_start + i) as u32;
            let b = (current_start + next_i) as u32;
            let c = (next_start + i) as u32;
            let d = (next_start + next_i) as u32;

            // Two triangles forming the quad.
            indices.extend_from_slice(&[a, c, b, b, c, d]);
        }
    }

    if !has_hollow {
        let last_start = ((layers.len() - 1) * layer_size) as u32;
        cap_layer(&layers[0], 0, true, &mut indices); // bottom, flipped winding  
        cap_layer(&layers[layers.len() - 1], last_start, false, &mut indices); // top  
    }

    Ok((vertices, indices))
}

fn cap_layer(layer: &MeshLayer, layer_start: u32, flip: bool, indices: &mut Vec<u32>) {
    let n = layer.0.len() as u32;
    if n < 3 {
        return;
    }
    for i in 1..n - 1 {
        let a = layer_start;
        let b = layer_start + i;
        let c = layer_start + i + 1;
        if flip {
            indices.extend_from_slice(&[a, c, b]);
        } else {
            indices.extend_from_slice(&[a, b, c]);
        }
    }
}

fn generate_normals(vertices: &[Vec3], indices: &[u32]) -> Vec<Vec3> {
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
        PathCurve::Circle | PathCurve::Circle2 | PathCurve::Test => {
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
        PathCurve::Line | PathCurve::Flexible => {
            let length = path_data.end - path_data.begin;
            let steps = path_data.curve.high_resolution_steps(); // same LOD lookup as circular  
            let step_size = length / steps;

            let mut z_offset = -0.5 + path_data.begin;
            let mut x_offset = path_data.shear_x * path_data.begin;
            let mut y_offset = path_data.shear_y * path_data.begin;

            let x_step = path_data.shear_x * length / steps;
            let y_step = path_data.shear_y * length / steps;

            let mut percent_of_path = path_data.begin;
            let percent_step = step_size * 0.999999;

            let mut step = 0;
            loop {
                let node_pos = Vec3::new(x_offset, y_offset, z_offset);

                let vertices = face_vertices.0.iter().map(|v| *v + node_pos).collect();
                layers.push(MeshLayer(vertices));

                if step as f32 >= steps {
                    break;
                }

                percent_of_path += percent_step;
                if percent_of_path > path_data.end {
                    break;
                }

                step += 1;
                x_offset += x_step;
                y_offset += y_step;
                z_offset += step_size;
            }

            Ok(layers)
        }
    }
}

fn build_profile(
    hollow_shape: HollowShape,
    hollow_size: f32,
    profile: ProfileShape,
    x_scale: f32,
    y_scale: f32,
) -> Result<MeshLayer, ObjectUpdateError> {
    let outer_vertices = build_profile_vertices(profile, x_scale, y_scale)?;
    let mut vertices = outer_vertices.clone();

    if hollow_size <= 0.0 {
        return Ok(MeshLayer(vertices));
    }

    let hollow_profile = match hollow_shape {
        HollowShape::Same => profile,
        HollowShape::Circle => ProfileShape::Circle,
        HollowShape::Square => ProfileShape::Square,
        HollowShape::Triangle => ProfileShape::EqualTriangle,
        HollowShape::Unknown => ProfileShape::Square,
    };

    let mut hollow_vertices = build_profile_vertices(hollow_profile, x_scale, y_scale)?;
    hollow_vertices.reverse();

    for vertex in hollow_vertices {
        vertices.push(vertex * hollow_size);
    }

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

        ProfileShape::HalfCircle => {
            for i in 0..sides {
                let angle = std::f32::consts::PI * i as f32 / (sides - 1) as f32;
                vertices.push(Vec3::new(angle.cos() * x_scale, angle.sin() * y_scale, 0.0));
            }
        }

        ProfileShape::Square | ProfileShape::IsoTriangle | ProfileShape::RightTriangle => {
            let scale = std::f32::consts::FRAC_1_SQRT_2;
            let corners = [(1.0, 0.0), (0.0, 1.0), (-1.0, 0.0), (0.0, -1.0)];
            for (x, y) in corners {
                vertices.push(Vec3::new(x * scale * x_scale, y * scale * y_scale, 0.0));
            }
        }

        ProfileShape::EqualTriangle => {
            let corners = [(1.0, 0.0), (-0.5, 0.8660254), (-0.5, -0.8660254)];
            for (x, y) in corners {
                vertices.push(Vec3::new(x * 0.5 * x_scale, y * 0.5 * y_scale, 0.0));
            }
        }

        ProfileShape::Unknown => {
            return Err(ObjectUpdateError::Unimplemented {
                feature: format!("Profile shape {profile:?}"),
            });
        }
    }

    Ok(vertices)
}

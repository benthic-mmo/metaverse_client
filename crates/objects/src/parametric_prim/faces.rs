use super::{
    geometry::append_hollow_cap,
    profile::{MeshLayer, hollow_profile_shape, profile_face_ranges, profile_vertex_count},
};
use crate::errors::ObjectUpdateError;
use benthic_default_asset_converter::default_texture_path;
use benthic_protocol::objects::types::{
    BlinnPhongMaterial, MaterialModel, RenderFace, TextureAnimation, TextureTransform,
};
use glam::Vec3;
use metaverse_messages::utils::{
    path::{HollowShape, ProfileShape},
    texture_entry::TextureEntries,
};
use std::{collections::HashMap, path::PathBuf};

fn generate_normals(vertices: &[Vec3], indices: &[u32]) -> Vec<Vec3> {
    let mut normals = vec![Vec3::ZERO; vertices.len()];

    for triangle in indices.as_chunks::<3>().0 {
        let a = triangle[0] as usize;
        let b = triangle[1] as usize;
        let c = triangle[2] as usize;

        let normal = (vertices[b] - vertices[a]).cross(vertices[c] - vertices[a]);

        normals[a] += normal;
        normals[b] += normal;
        normals[c] += normal;
    }

    for normal in &mut normals {
        *normal = normal.try_normalize().unwrap_or(Vec3::Z);
    }

    normals
}

fn face_texture(texture_paths: &HashMap<u32, PathBuf>, face_index: u32) -> Option<PathBuf> {
    texture_paths
        .get(&face_index)
        .or_else(|| texture_paths.get(&u32::MAX))
        .cloned()
}

fn make_face(
    face_index: u32,
    vertices: Vec<Vec3>,
    indices: Vec<u32>,
    uv: Vec<[f32; 2]>,
    texture_paths: &HashMap<u32, PathBuf>,
    texture: &Option<TextureEntries>,
    texture_animation: Option<TextureAnimation>,
) -> RenderFace {
    let normals = generate_normals(&vertices, &indices);

    let material = texture.as_ref().map(|entries| {
        let face = entries.faces.get(&face_index).unwrap_or(&entries.default);

        let tex_animation = texture_animation
            .as_ref()
            .and_then(|animation| match animation.face {
                f if f == face_index as i8 || f == -1 => Some(animation.clone()),
                _ => None,
            });

        MaterialModel::BlinnPhong(BlinnPhongMaterial {
            texture_path: face_texture(texture_paths, face_index)
                .unwrap_or_else(default_texture_path),
            bump: face.bumpmap,
            color: face.rgba,
            fullbright: face.fullbright,
            glow: face.glow,
            shiny: face.shiny,
            tex_gen: face.tex_gen,
            texture_transform: TextureTransform {
                repeat_u: face.repeat_u,
                repeat_v: face.repeat_v,
                offset_u: face.offset_u,
                offset_v: face.offset_v,
                rotation: face.rotation,
            },
            texture_animation: tex_animation,
        })
    });

    RenderFace {
        face_index,
        vertices,
        indices,
        uv,
        normals: Some(normals),
        weights: None,
        material,
    }
}

/// Extract one ring's faces from the path layers.
fn append_ring_faces(
    faces: &mut Vec<RenderFace>,
    layers: &[MeshLayer],
    profile: ProfileShape,
    ring_start: usize,
    ring_size: usize,
    texture_face_start: u32,
    texture_paths: &HashMap<u32, PathBuf>,
    texture: &Option<TextureEntries>,
    texture_animation: &Option<TextureAnimation>,
) {
    if layers.len() < 2 || ring_size < 2 {
        return;
    }

    let ranges = profile_face_ranges(profile, ring_size);
    let num_layers = layers.len();

    for (range_index, (start, count)) in ranges.into_iter().enumerate() {
        if count == 0 {
            continue;
        }

        let verts_per_layer = count + 1;
        let mut vertices = Vec::with_capacity(num_layers * verts_per_layer);
        let mut uv = Vec::with_capacity(num_layers * verts_per_layer);
        let mut indices = Vec::with_capacity((num_layers - 1) * count * 6);

        for (layer_index, layer) in layers.iter().enumerate() {
            for local_i in 0..=count {
                let ring_index = (start + local_i) % ring_size;
                let vertex_index = ring_start + ring_index;

                let Some(vertex) = layer.0.get(vertex_index) else {
                    return;
                };

                vertices.push(*vertex);

                let u = local_i as f32 / count as f32;
                let v = 1.0 - layer_index as f32 / (num_layers - 1) as f32;
                uv.push([u, v]);
            }
        }

        for layer_index in 0..num_layers - 1 {
            let current = (layer_index * verts_per_layer) as u32;
            let next = ((layer_index + 1) * verts_per_layer) as u32;

            for local_i in 0..count {
                let i = local_i as u32;

                let a = current + i;
                let b = current + i + 1;
                let c = next + i;
                let d = next + i + 1;

                // Preserve the existing outward-facing winding.
                indices.extend_from_slice(&[a, b, c, b, d, c]);
            }
        }

        let face_index = texture_face_start + range_index as u32;

        faces.push(make_face(
            face_index,
            vertices,
            indices,
            uv,
            texture_paths,
            texture,
            texture_animation.clone(),
        ));
    }
}

fn planar_uvs(vertices: &[Vec3]) -> Vec<[f32; 2]> {
    if vertices.is_empty() {
        return Vec::new();
    }

    let min_x = vertices.iter().map(|v| v.x).fold(f32::INFINITY, f32::min);
    let max_x = vertices
        .iter()
        .map(|v| v.x)
        .fold(f32::NEG_INFINITY, f32::max);
    let min_y = vertices.iter().map(|v| v.y).fold(f32::INFINITY, f32::min);
    let max_y = vertices
        .iter()
        .map(|v| v.y)
        .fold(f32::NEG_INFINITY, f32::max);

    let width = (max_x - min_x).max(f32::EPSILON);
    let height = (max_y - min_y).max(f32::EPSILON);

    vertices
        .iter()
        .map(|v| [(v.x - min_x) / width, 1.0 - (v.y - min_y) / height])
        .collect()
}

/// Splits path-generated geometry into separately textured faces.
pub(super) fn split_faces(
    layers: &[MeshLayer],
    profile: ProfileShape,
    hollow_shape: HollowShape,
    has_hollow: bool,
    texture_paths: &HashMap<u32, PathBuf>,
    texture: &Option<TextureEntries>,
    texture_animation: Option<TextureAnimation>,
) -> Result<Vec<RenderFace>, ObjectUpdateError> {
    if layers.len() < 2 {
        return Ok(Vec::new());
    }

    let outer_count = profile_vertex_count(profile);
    let inner_profile = hollow_profile_shape(hollow_shape, profile);
    let inner_count = if has_hollow {
        profile_vertex_count(inner_profile)
    } else {
        0
    };

    let expected_size = outer_count + inner_count;

    for layer in layers {
        if layer.0.len() != expected_size {
            return Err(ObjectUpdateError::Unimplemented {
                feature: format!(
                    "Profile layer has {} vertices; expected {}",
                    layer.0.len(),
                    expected_size,
                ),
            });
        }
    }

    // OpenSim face 0 is the top cap; side faces begin at 1.
    let outer_face_count = profile_face_ranges(profile, outer_count).len() as u32;
    let inner_face_count = if has_hollow {
        profile_face_ranges(inner_profile, inner_count).len() as u32
    } else {
        0
    };

    let is_round = matches!(profile, ProfileShape::Circle | ProfileShape::HalfCircle);
    let hollow_gap = if has_hollow && is_round { 1 } else { 0 };

    let outer_face_start = 1;
    let inner_face_start = outer_face_start + outer_face_count + hollow_gap;
    let bottom_face_index = inner_face_start + inner_face_count;

    let mut faces = Vec::new();

    append_ring_faces(
        &mut faces,
        layers,
        profile,
        0,
        outer_count,
        outer_face_start,
        texture_paths,
        texture,
        &texture_animation,
    );

    if has_hollow {
        append_ring_faces(
            &mut faces,
            layers,
            inner_profile,
            outer_count,
            inner_count,
            inner_face_start,
            texture_paths,
            texture,
            &texture_animation,
        );
    }

    let bottom = &layers[0];
    let top = &layers[layers.len() - 1];

    for (layer, flip, face_index) in [(bottom, true, bottom_face_index), (top, false, 0)] {
        let mut vertices = Vec::new();
        let mut indices = Vec::new();

        if has_hollow {
            append_hollow_cap(
                layer,
                outer_count,
                inner_count,
                flip,
                &mut vertices,
                &mut indices,
            );
        } else {
            vertices.extend_from_slice(&layer.0[..outer_count]);

            let count = outer_count as u32;
            for i in 1..count.saturating_sub(1) {
                if flip {
                    indices.extend_from_slice(&[0, i + 1, i]);
                } else {
                    indices.extend_from_slice(&[0, i, i + 1]);
                }
            }
        }

        let uv = planar_uvs(&vertices);

        faces.push(make_face(
            face_index,
            vertices,
            indices,
            uv,
            texture_paths,
            texture,
            texture_animation.clone(),
        ));
    }

    Ok(faces)
}

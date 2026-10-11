use std::{
    collections::HashMap,
    hash::{Hash, Hasher},
    path::{Path, PathBuf},
};

use benthic_protocol::{
    objects::{GeneratorObject, ParametricPrimData},
    render_data::{RenderFace, RenderObject},
    session::{cache_enabled, create_sub_object_dir},
};
use glam::{Quat, Vec3};
use metaverse_mesh::mesh::generate::generate_object_mesh;
use metaverse_messages::utils::{
    path::{HollowShape, PathCurve, PrimPath, ProfileShape},
    texture_entry::TextureEntries,
};
use rgb::Rgba;
use uuid::Uuid;

use crate::{
    errors::ObjectUpdateError,
    object_updates::{ObjectUpdateAction, RenderObjectData},
};

const MESH_GENERATOR_VERSION: u32 = 16;

/// One layer of vertices along a prim's path.
struct MeshLayer(Vec<Vec3>);
fn is_hollow_cylinder(object: &ParametricPrimData<TextureEntries, PrimPath>) -> bool {
    matches!(object.path_data.profile_shape, ProfileShape::Circle)
        && object.path_data.profile_hollow > 0.0
}
pub async fn handle_parametric_prim(
    out_dir: &Path,
    object: &ParametricPrimData<TextureEntries, PrimPath>,
    texture_paths: HashMap<u32, PathBuf>,
) -> Result<Vec<ObjectUpdateAction>, ObjectUpdateError> {
    let cache_key = create_cache_key(object);
    let base_dir = create_sub_object_dir(out_dir, &format!("{cache_key:016x}"))?;

    let profile = build_profile(
        object.path_data.hollow_shape,
        object.path_data.profile_hollow,
        object.path_data.profile_shape,
    )?;

    let layers = follow_path(object.path_data.clone(), &profile)?;
    let has_hollow = object.path_data.profile_hollow > 0.0;

    let debug_faces = is_hollow_cylinder(object);
    let faces = split_faces(
        &layers,
        object.path_data.profile_shape,
        object.path_data.hollow_shape,
        has_hollow,
        &texture_paths,
        &object.texture,
    )?;
    if debug_faces {
        println!(
            "\n=== Hollow cylinder {}: face assignments ===",
            object.local_id
        );

        for face in &faces {
            println!(
                "Face {:>2}: texture={:?}, color={:?}, vertices={}, triangles={}",
                face.face_index,
                face.texture,
                face.color,
                face.vertices.len(),
                face.indices.len() / 3,
            );
        }

        println!("=== End hollow cylinder faces ===\n");
    }

    let json_path = base_dir.join(format!("{cache_key}.json"));
    let glb_path = base_dir.join(format!("{cache_key}.glb"));

    let render_object = RenderObject {
        name: "Prim".to_string(),
        id: object.full_id,
        faces,
        skin: None,
    };

    std::fs::write(&json_path, serde_json::to_vec_pretty(&render_object)?)?;

    if !glb_path.exists() || !cache_enabled() {
        generate_object_mesh(json_path, glb_path.clone())?;
    }

    Ok(vec![ObjectUpdateAction::Render(RenderObjectData {
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
    })])
}

/// Build a cache key from geometry parameters, texture assignments and colors.
/// Build a cache key from geometry parameters, texture IDs, and colors.
fn create_cache_key(object: &ParametricPrimData<TextureEntries, PrimPath>) -> u64 {
    let mut hasher = std::collections::hash_map::DefaultHasher::new();

    object.path_data.hash(&mut hasher);
    MESH_GENERATOR_VERSION.hash(&mut hasher);

    if let Some(texture) = &object.texture {
        texture.default.texture_id.hash(&mut hasher);
        hash_color(texture.default.rgba, &mut hasher);

        let mut faces: Vec<_> = texture.faces.iter().collect();
        faces.sort_by_key(|(index, _)| **index);

        for (index, face) in faces {
            index.hash(&mut hasher);
            face.texture_id.hash(&mut hasher);
            hash_color(face.rgba, &mut hasher);
        }
    }

    hasher.finish()
}

fn hash_color<H: Hasher>(color: Rgba<u8>, hasher: &mut H) {
    (color.r, color.g, color.b, color.a).hash(hasher);
}

fn generate_normals(vertices: &[Vec3], indices: &[u32]) -> Vec<Vec3> {
    let mut normals = vec![Vec3::ZERO; vertices.len()];

    for triangle in indices.chunks_exact(3) {
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

fn linear_path_scale(scale: f32, t: f32) -> f32 {
    // Matches LLPathParams::getBeginScale/getEndScale.
    let begin = if scale > 1.0 { 2.0 - scale } else { 1.0 };
    let end = if scale < 1.0 { scale } else { 1.0 };

    begin + (end - begin) * t
}

fn taper_scale(taper: f32, t: f32) -> f32 {
    let mut begin = 1.0;
    let mut end = 1.0 - taper;

    if end > 1.0 {
        begin = 2.0 - end;
        end = 1.0;
    }

    begin + (end - begin) * t
}

fn follow_path(
    path_data: PrimPath,
    profile: &MeshLayer,
) -> Result<Vec<MeshLayer>, ObjectUpdateError> {
    let length = path_data.end - path_data.begin;

    if length <= f32::EPSILON {
        return Err(ObjectUpdateError::MeshTooSmall);
    }

    let step_count = path_data.curve.high_resolution_steps().max(1.0).ceil() as usize;
    let mut layers = Vec::with_capacity(step_count + 1);

    match path_data.curve {
        PathCurve::Line | PathCurve::Flexible => {
            for i in 0..=step_count {
                let fraction = i as f32 / step_count as f32;
                let t = path_data.begin + length * fraction;

                let scale_x = linear_path_scale(path_data.scale_x, t);
                let scale_y = linear_path_scale(path_data.scale_y, t);

                let offset = Vec3::new(path_data.shear_x * t, path_data.shear_y * t, t - 0.5);

                let vertices = profile
                    .0
                    .iter()
                    .map(|v| Vec3::new(v.x * scale_x, v.y * scale_y, v.z) + offset)
                    .collect();

                layers.push(MeshLayer(vertices));
            }
        }

        PathCurve::Circle | PathCurve::Circle2 | PathCurve::Test => {
            // The profile has a base radius of 0.5.
            let ring_radius = 0.5 * (1.0 - path_data.scale_y);

            for i in 0..=step_count {
                let fraction = i as f32 / step_count as f32;
                let t = path_data.begin + length * fraction;

                let angle =
                    std::f32::consts::TAU * path_data.revolutions * t - path_data.shear_y * 0.9;

                let offset = Vec3::new(0.0, angle.cos() * ring_radius, angle.sin() * ring_radius);

                let scale_x = path_data.scale_x * taper_scale(path_data.taper_x, t);
                let scale_y = path_data.scale_y * taper_scale(path_data.taper_y, t);
                let rotation = Quat::from_rotation_x(angle);

                let vertices = profile
                    .0
                    .iter()
                    .map(|v| {
                        let scaled = Vec3::new(v.x * scale_x, v.y * scale_y, v.z);
                        rotation * scaled + offset
                    })
                    .collect();

                layers.push(MeshLayer(vertices));
            }
        }
    }

    Ok(layers)
}

fn hollow_profile_shape(shape: HollowShape, profile: ProfileShape) -> ProfileShape {
    match shape {
        HollowShape::Same => profile,
        HollowShape::Circle => ProfileShape::Circle,
        HollowShape::Square => ProfileShape::Square,
        HollowShape::Triangle => ProfileShape::EqualTriangle,
        HollowShape::Unknown => ProfileShape::Square,
    }
}

fn build_profile(
    hollow_shape: HollowShape,
    hollow_size: f32,
    profile: ProfileShape,
) -> Result<MeshLayer, ObjectUpdateError> {
    let mut vertices = build_profile_vertices(profile)?;

    if hollow_size <= 0.0 {
        return Ok(MeshLayer(vertices));
    }

    let inner_profile = hollow_profile_shape(hollow_shape, profile);
    let mut inner = build_profile_vertices(inner_profile)?;

    // The inner perimeter runs in the opposite direction to the outer one.
    inner.reverse();

    let hollow_size = hollow_size.clamp(0.0, 1.0);
    vertices.extend(inner.into_iter().map(|v| v * hollow_size));

    Ok(MeshLayer(vertices))
}

fn build_profile_vertices(profile: ProfileShape) -> Result<Vec<Vec3>, ObjectUpdateError> {
    let sides = profile.default_sides_high();
    let mut vertices = Vec::with_capacity(sides);

    match profile {
        ProfileShape::Circle => {
            for i in 0..sides {
                let angle = std::f32::consts::TAU * i as f32 / sides as f32;
                vertices.push(Vec3::new(angle.cos() * 0.5, angle.sin() * 0.5, 0.0));
            }
        }

        ProfileShape::HalfCircle => {
            for i in 0..sides {
                let angle = std::f32::consts::PI * i as f32 / (sides - 1) as f32;
                vertices.push(Vec3::new(angle.cos() * 0.5, angle.sin() * 0.5, 0.0));
            }
        }

        ProfileShape::IsoTriangle => {
            for (x, y) in [(0.0, 0.5), (-0.4330127, -0.25), (0.4330127, -0.25)] {
                vertices.push(Vec3::new(x, y, 0.0));
            }
        }

        ProfileShape::RightTriangle => {
            vertices.extend_from_slice(&[
                Vec3::new(-0.5, -0.5, 0.0),
                Vec3::new(0.5, -0.5, 0.0),
                Vec3::new(-0.5, 0.5, 0.0),
            ]);
        }

        ProfileShape::Square => {
            vertices.extend_from_slice(&[
                Vec3::new(-0.5, -0.5, 0.0),
                Vec3::new(0.5, -0.5, 0.0),
                Vec3::new(0.5, 0.5, 0.0),
                Vec3::new(-0.5, 0.5, 0.0),
            ]);
        }

        ProfileShape::EqualTriangle => {
            for (x, y) in [(1.0, 0.0), (-0.5, 0.8660254), (-0.5, -0.8660254)] {
                vertices.push(Vec3::new(x * 0.53, y * 0.53, 0.0));
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

fn profile_vertex_count(profile: ProfileShape) -> usize {
    match profile {
        ProfileShape::Square => 4,
        ProfileShape::IsoTriangle | ProfileShape::RightTriangle | ProfileShape::EqualTriangle => 3,
        ProfileShape::Circle | ProfileShape::HalfCircle => profile.default_sides_high(),
        ProfileShape::Unknown => 0,
    }
}

/// Returns the start vertex and segment count for each logical face.
fn profile_face_ranges(profile: ProfileShape, ring_size: usize) -> Vec<(usize, usize)> {
    if ring_size < 2 {
        return Vec::new();
    }

    match profile {
        ProfileShape::Circle => vec![(0, ring_size)],
        ProfileShape::HalfCircle
        | ProfileShape::Square
        | ProfileShape::IsoTriangle
        | ProfileShape::RightTriangle
        | ProfileShape::EqualTriangle => (0..ring_size).map(|i| (i, 1)).collect(),
        ProfileShape::Unknown => Vec::new(),
    }
}

fn face_texture(texture_paths: &HashMap<u32, PathBuf>, face_index: u32) -> Option<PathBuf> {
    texture_paths
        .get(&face_index)
        .or_else(|| texture_paths.get(&u32::MAX))
        .cloned()
}

fn face_color(texture: &Option<TextureEntries>, face_index: u32) -> Rgba<u8> {
    texture
        .as_ref()
        .map(|entries| {
            entries
                .faces
                .get(&face_index)
                .map(|face| face.rgba)
                .unwrap_or(entries.default.rgba)
        })
        .unwrap_or_else(|| Rgba::new(255, 255, 255, 255))
}

/// Construct a RenderFace and calculate its normals consistently.
fn make_face(
    face_index: u32,
    vertices: Vec<Vec3>,
    indices: Vec<u32>,
    uv: Vec<[f32; 2]>,
    texture_paths: &HashMap<u32, PathBuf>,
    texture: &Option<TextureEntries>,
) -> RenderFace {
    let normals = generate_normals(&vertices, &indices);

    RenderFace {
        face_index,
        vertices,
        indices,
        uv,
        normals: Some(normals),
        texture: face_texture(texture_paths, face_index),
        weights: None,
        color: face_color(texture, face_index),
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
                let (u, v) = match profile {
                    ProfileShape::Circle | ProfileShape::HalfCircle => (
                        1.0 - layer_index as f32 / (num_layers - 1) as f32,
                        local_i as f32 / count as f32,
                    ),
                    _ => (
                        local_i as f32 / count as f32,
                        1.0 - layer_index as f32 / (num_layers - 1) as f32,
                    ),
                };
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

/// Split path-generated geometry into separately textured faces.
fn split_faces(
    layers: &[MeshLayer],
    profile: ProfileShape,
    hollow_shape: HollowShape,
    has_hollow: bool,
    texture_paths: &HashMap<u32, PathBuf>,
    texture: &Option<TextureEntries>,
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
        ));
    }

    Ok(faces)
}

/// Resample a closed polygonal ring evenly by perimeter distance.
fn resample_ring(vertices: &[Vec3], count: usize) -> Vec<Vec3> {
    if vertices.is_empty() || count == 0 {
        return Vec::new();
    }

    if vertices.len() == 1 {
        return vec![vertices[0]; count];
    }

    let edge_lengths: Vec<f32> = (0..vertices.len())
        .map(|i| vertices[i].distance(vertices[(i + 1) % vertices.len()]))
        .collect();

    let perimeter: f32 = edge_lengths.iter().sum();

    if perimeter <= f32::EPSILON {
        return vec![vertices[0]; count];
    }

    let mut result = Vec::with_capacity(count);

    for sample in 0..count {
        let target = perimeter * sample as f32 / count as f32;
        let mut distance = 0.0;

        for (i, &edge_length) in edge_lengths.iter().enumerate() {
            let next_distance = distance + edge_length;

            if target <= next_distance || i == edge_lengths.len() - 1 {
                let t = if edge_length > f32::EPSILON {
                    ((target - distance) / edge_length).clamp(0.0, 1.0)
                } else {
                    0.0
                };

                let a = vertices[i];
                let b = vertices[(i + 1) % vertices.len()];
                result.push(a.lerp(b, t));
                break;
            }

            distance = next_distance;
        }
    }

    result
}

/// Create an annular cap between outer and inner profile rings.
fn append_hollow_cap(
    layer: &MeshLayer,
    outer_count: usize,
    inner_count: usize,
    flip: bool,
    vertices: &mut Vec<Vec3>,
    indices: &mut Vec<u32>,
) {
    if outer_count < 3 || inner_count < 3 || layer.0.len() < outer_count + inner_count {
        return;
    }

    let outer = &layer.0[..outer_count];
    let inner: Vec<Vec3> = layer.0[outer_count..outer_count + inner_count]
        .iter()
        .rev()
        .copied()
        .collect();

    let count = outer_count.max(inner_count);
    let outer = resample_ring(outer, count);
    let inner = resample_ring(&inner, count);

    if outer.len() != count || inner.len() != count {
        return;
    }

    vertices.extend_from_slice(&outer);
    vertices.extend_from_slice(&inner);

    let inner_start = count as u32;

    for i in 0..count {
        let next = ((i + 1) % count) as u32;
        let outer_a = i as u32;
        let outer_b = next;
        let inner_a = inner_start + i as u32;
        let inner_b = inner_start + next;

        if flip {
            indices.extend_from_slice(&[outer_a, inner_a, outer_b, outer_b, inner_a, inner_b]);
        } else {
            indices.extend_from_slice(&[outer_a, outer_b, inner_a, outer_b, inner_b, inner_a]);
        }
    }
}

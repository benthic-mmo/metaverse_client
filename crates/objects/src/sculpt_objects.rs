use std::path::PathBuf;

use benthic_protocol::{
    objects::{GeneratorObject, SculptObjectData},
    render_data::{RenderFace, RenderObject},
    session::cache_enabled,
};
use glam::Vec3;
use metaverse_mesh::mesh::generate::generate_object_mesh;
use metaverse_messages::{
    http::scene::SculptType, udp::object::object_update::SculptData,
    utils::texture_entry::TextureEntries,
};
use uuid::Uuid;

use crate::{
    errors::ObjectUpdateError,
    object_updates::{ObjectUpdateAction, RenderObjectData},
};

pub async fn handle_sculpt_object(
    out_dir: &PathBuf,
    texture_path: PathBuf,
    object: SculptObjectData<TextureEntries, SculptData>,
) -> Result<Vec<ObjectUpdateAction>, ObjectUpdateError> {
    let image = image::open(&texture_path)?.into_rgb8();
    let (width, height) = image.dimensions();
    let mut actions = Vec::new();

    println!("Texture dimensions: {width}x{height}");

    let mut vertices: Vec<Vec3> = Vec::with_capacity((width * height) as usize);

    for y in 0..height {
        for x in 0..width {
            let pixel = image.get_pixel(x, y);

            vertices.push(Vec3::new(
                pixel[0] as f32 / 255.0 - 0.5,
                pixel[1] as f32 / 255.0 - 0.5,
                pixel[2] as f32 / 255.0 - 0.5,
            ));
        }
    }

    let w = width as usize;
    let h = height as usize;

    if w < 2 || h < 2 {
        Err(ObjectUpdateError::MeshTooSmall)?;
    }

    // Reshape the row-major vertices into horizontal scanlines.
    // this is just how you do it I guess
    let mut rows: Vec<Vec<Vec3>> = vertices
        .chunks_exact(w)
        .map(|chunk| chunk.to_vec())
        .collect();

    let original_width = w;

    match object.data.sculpt_type {
        SculptType::Plane => {}

        SculptType::Sphere => {
            // Close the seam.
            if rows.len().is_multiple_of(2) {
                for row in &mut rows {
                    row.push(row[0]);
                }
            } else {
                for row in &mut rows {
                    let last = row.len() - 1;
                    row[0] = row[last];
                }
            }

            // Collapse or extend the poles.
            let mid = original_width / 2;
            let top_pole = rows[0][mid];
            let bottom_pole = rows[rows.len() - 1][mid];
            let row_width = rows[0].len();

            if rows.len().is_multiple_of(2) {
                rows.insert(0, vec![top_pole; row_width]);
                rows.push(vec![bottom_pole; row_width]);
            } else {
                let last = rows.len() - 1;

                for x in 0..row_width {
                    rows[0][x] = top_pole;
                    rows[last][x] = bottom_pole;
                }
            }
        }

        SculptType::Torus => {
            // Close the first loop.
            if rows.len().is_multiple_of(2) {
                for row in &mut rows {
                    row.push(row[0]);
                }
            } else {
                for row in &mut rows {
                    let last = row.len() - 1;
                    row[0] = row[last];
                }
            }

            // Close the second loop.
            rows.push(rows[0].clone());
        }

        _ => {
            // Close the seam for other sculpt types.
            if rows.len().is_multiple_of(2) {
                for row in &mut rows {
                    row.push(row[0]);
                }
            } else {
                for row in &mut rows {
                    let last = row.len() - 1;
                    row[0] = row[last];
                }
            }
        }
    }

    let indices = create_sculpt_indices(&rows, false);

    let vertices: Vec<Vec3> = rows.into_iter().flatten().collect();

    let normals = create_sculpt_normals(&vertices, &indices);

    let file_path = out_dir.join(format!("{}.json", object.data.texture_id));
    let glb_path = out_dir.join(format!("{}.glb", object.data.texture_id));
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
        base_dir: out_dir.clone(),
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

fn create_sculpt_indices(rows: &[Vec<Vec3>], invert: bool) -> Vec<u32> {
    if rows.len() < 2 || rows[0].len() < 2 {
        return Vec::new();
    }

    let width = rows[0].len();

    assert!(
        rows.iter().all(|row| row.len() == width),
        "Sculpt rows must all have the same width"
    );

    let mut indices = Vec::with_capacity((rows.len() - 1) * (width - 1) * 6);

    for y in 1..rows.len() {
        for x in 1..width {
            let p4 = (y * width + x) as u32;
            let p3 = p4 - 1;
            let p2 = p4 - width as u32;
            let p1 = p3 - width as u32;

            if invert {
                indices.extend_from_slice(&[p1, p4, p3, p1, p2, p4]);
            } else {
                indices.extend_from_slice(&[p1, p3, p4, p1, p4, p2]);
            }
        }
    }

    indices
}

fn create_sculpt_normals(vertices: &[Vec3], indices: &[u32]) -> Vec<Vec3> {
    let mut normals = vec![Vec3::ZERO; vertices.len()];

    for triangle in indices.chunks_exact(3) {
        let a = triangle[0] as usize;
        let b = triangle[1] as usize;
        let c = triangle[2] as usize;

        let edge_a = vertices[b] - vertices[a];
        let edge_b = vertices[c] - vertices[a];
        let face_normal = edge_a.cross(edge_b);

        if face_normal.length_squared() <= f32::EPSILON {
            continue;
        }

        // The unnormalized cross product weights by triangle area.
        normals[a] += face_normal;
        normals[b] += face_normal;
        normals[c] += face_normal;
    }

    for normal in &mut normals {
        *normal = normal.normalize_or_zero();
    }

    normals
}

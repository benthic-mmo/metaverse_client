use std::{collections::HashMap, path::PathBuf};

use benthic_default_asset_converter::default_texture_path;
use benthic_protocol::{
    objects::{GeneratorObject, SculptObjectData},
    render_data::{BlinnPhongMaterial, MaterialModel, RenderFace, RenderObject, TextureTransform},
    session::{cache_enabled, create_sub_object_dir},
};
use glam::Vec3;
use metaverse_mesh::mesh::generate::generate_object_mesh;
use metaverse_messages::{
    http::scene::SculptType, udp::object::object_update::SculptData,
    utils::texture_entry::TextureEntries,
};

use crate::{
    errors::ObjectUpdateError,
    object_updates::{ObjectUpdateAction, RenderObjectData},
};

pub async fn handle_sculpt_object(
    out_dir: PathBuf,
    texture_path: PathBuf,
    object: &SculptObjectData<TextureEntries, SculptData>,
    texture_paths: HashMap<u32, PathBuf>,
) -> Result<Vec<ObjectUpdateAction>, ObjectUpdateError> {
    let base_dir = create_sub_object_dir(&out_dir, &object.data.texture_id.to_string())?;
    let file_path = base_dir.join(format!("{}.json", object.data.texture_id));
    let glb_path = base_dir.join(format!("{}.glb", object.data.texture_id));

    if !glb_path.exists() || !cache_enabled() {
        if !file_path.exists() || !cache_enabled() {
            let image = image::open(&texture_path)?.into_rgb8();
            let (width, height) = image.dimensions();

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
                return Err(ObjectUpdateError::MeshTooSmall);
            }

            // Reshape the row-major vertices into horizontal scanlines.
            let mut rows: Vec<Vec<Vec3>> = vertices
                .chunks_exact(w)
                .map(|chunk| chunk.to_vec())
                .collect();

            if object.data.mirror {
                for row in &mut rows {
                    for v in row.iter_mut() {
                        v.x = -v.x;
                    }
                }
            }

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
                    let row_width = rows[0].len();
                    let top_pole = rows[0][mid.min(row_width - 1)];
                    let bottom_pole = rows[rows.len() - 1][mid.min(row_width - 1)];
                    let last = rows.len() - 1;

                    for x in 0..row_width {
                        rows[0][x] = top_pole;
                        rows[last][x] = bottom_pole;
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
            // mirror negates X geometry AND flips handedness; invert flips handedness only.
            // XOR = reverse horizontal texture direction (sculpt_reverse_horizontal).
            let reverse_h = object.data.invert != object.data.mirror;
            let indices = create_sculpt_indices(&rows, reverse_h);
            let row_count = rows.len();
            let row_width = rows[0].len();

            let mut vertices = Vec::with_capacity(row_count * row_width);
            let mut uv = Vec::with_capacity(row_count * row_width);

            for (y, row) in rows.iter().enumerate() {
                for (x, _) in row.iter().enumerate() {
                    let vx = if reverse_h { row_width - 1 - x } else { x };
                    vertices.push(row[vx]);

                    uv.push([
                        1.0 - vx as f32 / (row_width - 1) as f32, // global mirror + flag-aware u
                        y as f32 / (row_count - 1) as f32,
                    ]);
                }
            }

            let mut normals = create_sculpt_normals(&vertices, &indices);
            if object.data.invert {
                for n in &mut normals {
                    *n = -*n;
                }
            }

            smooth_sculpt_normals(&mut normals, row_width, row_count, &object.data.sculpt_type);

            let material = if let Some(texture) = &object.texture {
                let face = &texture.default;
                Some(MaterialModel::BlinnPhong(BlinnPhongMaterial {
                    texture_path: texture_paths
                        .get(&u32::MAX)
                        .unwrap_or(&default_texture_path())
                        .clone(),
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
                }))
            } else {
                None
            };

            let render_object = RenderObject {
                name: "Prim".to_string(),
                id: object.full_id,
                attachment_data: None,
                faces: vec![RenderFace {
                    face_index: 0,
                    vertices,
                    indices,
                    uv,
                    normals: Some(normals),
                    weights: None,
                    material,
                }],
                skin: None,
            };

            std::fs::write(&file_path, serde_json::to_vec_pretty(&render_object)?)?;
        }

        generate_object_mesh(file_path.clone(), glb_path.clone())?;
    }

    Ok(vec![ObjectUpdateAction::Render(RenderObjectData {
        mesh_path: Some(glb_path),
        asset_id: object.full_id,
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

fn smooth_sculpt_normals(normals: &mut [Vec3], w: usize, h: usize, ty: &SculptType) {
    let set_avg = |n: &mut [Vec3], idxs: &[usize]| {
        let avg = idxs.iter().fold(Vec3::ZERO, |a, &i| a + n[i]);
        for &i in idxs {
            n[i] = avg;
        }
    };

    if matches!(ty, SculptType::Sphere) {
        let top: Vec<usize> = (0..w).collect();
        let bot: Vec<usize> = (0..w).map(|x| (h - 1) * w + x).collect();
        set_avg(normals, &top);
        set_avg(normals, &bot);
    }
    if matches!(
        ty,
        SculptType::Sphere | SculptType::Torus | SculptType::Cylinder
    ) {
        for y in 0..h {
            let (a, b) = (y * w, y * w + w - 1);
            let n = normals[a] + normals[b];
            normals[a] = n;
            normals[b] = n;
        }
    }
    if matches!(ty, SculptType::Torus) {
        for x in 0..w {
            let (a, b) = (x, (h - 1) * w + x);
            let n = normals[a] + normals[b];
            normals[a] = n;
            normals[b] = n;
        }
    }
    for n in normals.iter_mut() {
        *n = n.normalize_or_zero();
    }
}

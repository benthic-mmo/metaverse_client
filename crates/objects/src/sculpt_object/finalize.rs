use super::geometry::SculptGeometry;
use benthic_default_asset_converter::default_texture_path;
use benthic_protocol::objects::{
    objects::SculptObjectData,
    types::{BlinnPhongMaterial, MaterialModel, RenderFace, RenderObject, TextureTransform},
};
use glam::Vec3;
use metaverse_messages::{
    http::scene::SculptType, udp::object::object_update::SculptData,
    utils::texture_entry::TextureEntries,
};
use std::{collections::HashMap, path::PathBuf};
use uuid::Uuid;

pub(super) fn finalize_sculpt_object(
    object: &SculptObjectData<TextureEntries, SculptData>,
    texture_paths: &HashMap<u32, PathBuf>,
    geometry: SculptGeometry,
    asset_id: Uuid,
) -> RenderObject {
    let indices = create_sculpt_indices(&geometry.rows, geometry.reverse_h);

    let row_count = geometry.rows.len();
    let row_width = geometry.rows[0].len();

    let mut normals = create_sculpt_normals(&geometry.vertices, &indices);

    if object.data.invert {
        for normal in &mut normals {
            *normal = -*normal;
        }
    }

    smooth_sculpt_normals(&mut normals, row_width, row_count, &object.data.sculpt_type);

    let material = build_sculpt_material(object, texture_paths);

    RenderObject {
        name: "Prim".to_string(),
        id: asset_id,
        attachment_data: None,
        faces: vec![RenderFace {
            face_index: 0,
            vertices: geometry.vertices,
            indices,
            uv: geometry.uv,
            normals: Some(normals),
            weights: None,
            material,
        }],
        skin: None,
    }
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

    for triangle in indices.as_chunks::<3>().0 {
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

fn smooth_sculpt_normals(normals: &mut [Vec3], w: usize, h: usize, sculpt_type: &SculptType) {
    let set_avg = |normals: &mut [Vec3], indices: &[usize]| {
        let avg = indices.iter().fold(Vec3::ZERO, |sum, &i| sum + normals[i]);

        for &i in indices {
            normals[i] = avg;
        }
    };

    if matches!(sculpt_type, SculptType::Sphere) {
        let top: Vec<usize> = (0..w).collect();
        let bottom: Vec<usize> = (0..w).map(|x| (h - 1) * w + x).collect();

        set_avg(normals, &top);
        set_avg(normals, &bottom);
    }

    if matches!(
        sculpt_type,
        SculptType::Sphere | SculptType::Torus | SculptType::Cylinder
    ) {
        for y in 0..h {
            let (a, b) = (y * w, y * w + w - 1);
            let n = normals[a] + normals[b];

            normals[a] = n;
            normals[b] = n;
        }
    }

    if matches!(sculpt_type, SculptType::Torus) {
        for x in 0..w {
            let (a, b) = (x, (h - 1) * w + x);
            let n = normals[a] + normals[b];

            normals[a] = n;
            normals[b] = n;
        }
    }

    for normal in normals.iter_mut() {
        *normal = normal.normalize_or_zero();
    }
}

fn build_sculpt_material(
    object: &SculptObjectData<TextureEntries, SculptData>,
    texture_paths: &HashMap<u32, PathBuf>,
) -> Option<MaterialModel> {
    let face = &object.texture.as_ref()?.default;

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
        texture_animation: object.texture_animation.clone(),
    }))
}

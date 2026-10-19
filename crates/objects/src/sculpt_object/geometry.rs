use glam::Vec3;
use metaverse_messages::http::scene::SculptType;

use crate::errors::ObjectUpdateError;

pub(super) struct SculptGeometry {
    pub vertices: Vec<Vec3>,
    pub uv: Vec<[f32; 2]>,
    pub rows: Vec<Vec<Vec3>>,
    pub reverse_h: bool,
}

pub(super) fn build_sculpt_geometry(
    image: &image::RgbImage,
    sculpt_type: &SculptType,
    mirror: bool,
    invert: bool,
) -> Result<SculptGeometry, ObjectUpdateError> {
    let (width, height) = image.dimensions();
    let w = width as usize;
    let h = height as usize;

    if w < 2 || h < 2 {
        return Err(ObjectUpdateError::MeshTooSmall);
    }

    let mut rows: Vec<Vec<Vec3>> = (0..h)
        .map(|y| {
            (0..w)
                .map(|x| {
                    let pixel = image.get_pixel(x as u32, y as u32);

                    Vec3::new(
                        pixel[0] as f32 / 255.0 - 0.5,
                        pixel[1] as f32 / 255.0 - 0.5,
                        pixel[2] as f32 / 255.0 - 0.5,
                    )
                })
                .collect()
        })
        .collect();

    if mirror {
        for row in &mut rows {
            for vertex in row {
                vertex.x = -vertex.x;
            }
        }
    }

    let original_width = w;

    match sculpt_type {
        SculptType::Plane => {}

        SculptType::Sphere => {
            close_horizontal_seam(&mut rows);

            let mid = original_width / 2;
            let row_width = rows[0].len();
            let top_pole = rows[0][mid.min(row_width - 1)];
            let last = rows.len() - 1;
            let bottom_pole = rows[last][mid.min(row_width - 1)];

            for x in 0..row_width {
                rows[0][x] = top_pole;
                rows[last][x] = bottom_pole;
            }
        }

        SculptType::Torus => {
            close_horizontal_seam(&mut rows);
            rows.push(rows[0].clone());
        }

        _ => close_horizontal_seam(&mut rows),
    }

    // Preserve the existing horizontal texture direction and winding behavior.
    let reverse_h = invert != mirror;

    let row_count = rows.len();
    let row_width = rows[0].len();

    let mut vertices = Vec::with_capacity(row_count * row_width);
    let mut uv = Vec::with_capacity(row_count * row_width);

    for (y, row) in rows.iter().enumerate() {
        for x in 0..row_width {
            let vx = if reverse_h { row_width - 1 - x } else { x };

            vertices.push(row[vx]);

            uv.push([
                1.0 - vx as f32 / (row_width - 1) as f32,
                y as f32 / (row_count - 1) as f32,
            ]);
        }
    }

    Ok(SculptGeometry {
        vertices,
        uv,
        rows,
        reverse_h,
    })
}

fn close_horizontal_seam(rows: &mut [Vec<Vec3>]) {
    for row in rows {
        if row.len().is_multiple_of(2) {
            row.push(row[0]);
        } else {
            let last = row.len() - 1;
            row[0] = row[last];
        }
    }
}

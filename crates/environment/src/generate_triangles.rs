use crate::land::Land;
use glam::Vec3;
use metaverse_messages::utils::render_data::RenderObject;
use std::collections::HashMap;
use uuid::Uuid;

/// Generate a terrain patch with shared vertices and indexed triangles.
///
/// Includes edge stitching with north, east, and top-corner patches.
pub fn generate_mesh_with_indices(
    layer: &Land,
    north_layer: &Land,
    east_layer: &Land,
    top_corner: &Land,
) -> RenderObject {
    let scale = 1.0;
    let grid_size = layer.terrain_header.patch_size;

    let mut vertices: Vec<Vec3> = Vec::new();
    let mut indices: Vec<u16> = Vec::new();
    let mut vertex_map: HashMap<(i32, i32, i32), u16> = HashMap::new();

    // Helper closure for deduplicating vertices
    let mut add_vertex = |v: Vec3| -> u16 {
        // Convert position to integer key for hashing
        let key = (
            (v.x * 1_000.0) as i32,
            (v.y * 1_000.0) as i32,
            (v.z * 1_000.0) as i32,
        );
        if let Some(&i) = vertex_map.get(&key) {
            i
        } else {
            let i = vertices.len() as u16;
            vertices.push(v);
            vertex_map.insert(key, i);
            i
        }
    };

    for row in 0..grid_size - 1 {
        for col in 0..grid_size - 1 {
            // Base terrain vertices
            let top_left = row * grid_size + col;
            let top_right = top_left + 1;
            let bottom_left = (row + 1) * grid_size + col;
            let bottom_right = bottom_left + 1;

            let v0 = Vec3::new(col as f32, layer.heightmap[top_left as usize], row as f32);
            let v1 = Vec3::new(
                (col + 1) as f32,
                layer.heightmap[top_right as usize],
                row as f32,
            );
            let v2 = Vec3::new(
                col as f32,
                layer.heightmap[bottom_left as usize],
                (row + 1) as f32,
            );
            let v3 = Vec3::new(
                (col + 1) as f32,
                layer.heightmap[bottom_right as usize],
                (row + 1) as f32,
            );

            let i0 = add_vertex(v0);
            let i1 = add_vertex(v1);
            let i2 = add_vertex(v2);
            let i3 = add_vertex(v3);

            // Two triangles per quad
            indices.extend_from_slice(&[i0, i2, i1]);
            indices.extend_from_slice(&[i1, i2, i3]);

            // Stitching
            stitch_patches_with_indices(
                layer,
                north_layer,
                east_layer,
                top_corner,
                &mut add_vertex,
                &mut indices,
                col,
                row,
                scale,
            );
        }
    }

    RenderObject {
        vertices,
        indices,
        id: Uuid::nil(),
        name: layer.terrain_header.location.to_string(),
        skin: None,
        texture: None,
        uv: None,
    }
}

/// Stitch patch edges together using indices.
fn stitch_patches_with_indices<F>(
    layer: &Land,
    north_layer: &Land,
    east_layer: &Land,
    top_corner: &Land,
    add_vertex: &mut F,
    indices: &mut Vec<u16>,
    col: u8,
    row: u8,
    scale: f32,
) where
    F: FnMut(Vec3) -> u16,
{
    let grid_size = layer.terrain_header.patch_size;

    // Base layer
    let top_left = row * grid_size + col;
    let top_right = top_left + 1;
    let bottom_right = (row + 1) * grid_size + col + 1;

    let v0 = Vec3::new(
        col as f32,
        layer.heightmap[top_left as usize] * scale,
        row as f32,
    );
    let v1 = Vec3::new(
        (col + 1) as f32,
        layer.heightmap[top_right as usize] * scale,
        row as f32,
    );
    let v3 = Vec3::new(
        (col + 1) as f32,
        layer.heightmap[bottom_right as usize] * scale,
        (row + 1) as f32,
    );

    // North patch
    let top_left_north = (grid_size - 1) * grid_size + col;
    let top_right_north = top_left_north + 1;

    let n0 = Vec3::new(
        col as f32,
        north_layer.heightmap[top_left_north as usize] * scale,
        row as f32 - 1.0,
    );
    let n1 = Vec3::new(
        (col + 1) as f32,
        north_layer.heightmap[top_right_north as usize] * scale,
        row as f32 - 1.0,
    );

    // East patch
    let top_left_east = grid_size * row;
    let bottom_left_east = grid_size * (row + 1);

    let e0 = Vec3::new(
        (col as f32 + 2.0) * scale,
        east_layer.heightmap[top_left_east as usize] * scale,
        row as f32,
    );
    let e1 = Vec3::new(
        (col as f32 + 2.0) * scale,
        east_layer.heightmap[bottom_left_east as usize] * scale,
        (row + 1) as f32,
    );

    // Top-corner patch
    let top_corner_coord = grid_size * (grid_size - 1);
    let c0 = Vec3::new(
        (col as f32 + 2.0) * scale,
        top_corner.heightmap[top_corner_coord as usize] * scale,
        row as f32 - 1.0,
    );

    // Create stitching triangles using indices
    if row == 0 {
        let i_n0 = add_vertex(n0);
        let i_n1 = add_vertex(n1);
        let i_v0 = add_vertex(v0);
        let i_v1 = add_vertex(v1);

        indices.extend_from_slice(&[i_n0, i_v0, i_n1]);
        indices.extend_from_slice(&[i_n1, i_v0, i_v1]);
    }

    if col == grid_size - 2 {
        let i_v1 = add_vertex(v1);
        let i_v3 = add_vertex(v3);
        let i_e0 = add_vertex(e0);
        let i_e1 = add_vertex(e1);

        indices.extend_from_slice(&[i_v1, i_v3, i_e0]);
        indices.extend_from_slice(&[i_e0, i_v3, i_e1]);
    }

    if col == grid_size - 2 && row == 0 {
        let i_n1 = add_vertex(n1);
        let i_v1 = add_vertex(v1);
        let i_e0 = add_vertex(e0);
        let i_c0 = add_vertex(c0);

        indices.extend_from_slice(&[i_n1, i_v1, i_c0]);
        indices.extend_from_slice(&[i_c0, i_v1, i_e0]);
    }
}

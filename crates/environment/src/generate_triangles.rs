use glam::Vec3;

use crate::land::Land;

/// Generate the triangles of the patch's land data.
/// Requires the north layer, east layer and top corner in order to preform stitching on the
/// patches. The raw layer data comes in with a gap between tiles, for the viewer to stitch
/// together manually.
pub fn generate_triangles(
    layer: &Land,
    north_layer: &Land,
    east_layer: &Land,
    top_corner: &Land,
) -> Vec<Vec3> {
    let scale = 1.0;
    let mut triangles: Vec<Vec3> = Vec::new();
    let grid_size = layer.terrain_header.patch_size;
    for row in 0..grid_size - 1 {
        for col in 0..grid_size - 1 {
            let top_left = row * grid_size + col;
            let top_right = top_left + 1;
            let bottom_left = (row + 1) * grid_size + col;
            let bottom_right = bottom_left + 1;

            let x0 = col as f32 * scale;
            let y0 = layer.heightmap[top_left as usize] * scale;
            let z0 = row as f32 * scale;

            let x1 = (col + 1) as f32 * scale;
            let y1 = layer.heightmap[top_right as usize] * scale;
            let z1 = row as f32 * scale;

            let x2 = col as f32 * scale;
            let y2 = layer.heightmap[bottom_left as usize] * scale;
            let z2 = (row + 1) as f32 * scale;

            let x3 = (col + 1) as f32 * scale;
            let y3 = layer.heightmap[bottom_right as usize] * scale;
            let z3 = (row + 1) as f32 * scale;

            triangles.push(Vec3 {
                x: x0,
                y: y0,
                z: z0,
            });
            triangles.push(Vec3 {
                x: x2,
                y: y2,
                z: z2,
            });
            triangles.push(Vec3 {
                x: x1,
                y: y1,
                z: z1,
            });
            triangles.push(Vec3 {
                x: x1,
                y: y1,
                z: z1,
            });
            triangles.push(Vec3 {
                x: x2,
                y: y2,
                z: z2,
            });
            triangles.push(Vec3 {
                x: x3,
                y: y3,
                z: z3,
            });
            // create the stitch geometry
            stitch_patches(
                layer,
                north_layer,
                east_layer,
                top_corner,
                &mut triangles,
                col,
                row,
                scale,
            );
        }
    }
    triangles
}

fn stitch_patches(
    layer: &Land,
    north_layer: &Land,
    east_layer: &Land,
    top_corner: &Land,
    triangles: &mut Vec<Vec3>,
    col: u8,
    row: u8,
    scale: f32,
) {
    let grid_size = layer.terrain_header.patch_size;
    let top_left = row * grid_size + col;
    let top_right = top_left + 1;
    let bottom_left = (row + 1) * grid_size + col;
    let bottom_right = bottom_left + 1;

    let x0 = col as f32 * scale;
    let y0 = layer.heightmap[top_left as usize] * scale;
    let z0 = row as f32 * scale;

    let x1 = (col + 1) as f32 * scale;
    let y1 = layer.heightmap[top_right as usize] * scale;
    let z1 = row as f32 * scale;

    let x3 = (col + 1) as f32 * scale;
    let y3 = layer.heightmap[bottom_right as usize] * scale;
    let z3 = (row + 1) as f32 * scale;

    let top_left_north = (grid_size - 1) * grid_size + col;
    let top_right_north = top_left_north + 1;

    let north_x0 = col as f32 * scale;
    let north_y0 = north_layer.heightmap[top_left_north as usize] * scale;
    let north_z0 = (row as f32 - 1.0) * scale;

    let north_x1 = (col + 1) as f32 * scale;
    let north_y1 = north_layer.heightmap[top_right_north as usize] * scale;
    let north_z1 = (row as f32 - 1.0) * scale;
    let top_left_east = grid_size * row;
    let bottom_left_east = grid_size * (row + 1);

    let east_x0 = (col as f32 + 2.0) * scale;
    let east_y0 = east_layer.heightmap[top_left_east as usize] * scale;
    let east_z0 = row as f32 * scale;

    let east_x1 = (col as f32 + 2.0) * scale;
    let east_y1 = east_layer.heightmap[bottom_left_east as usize] * scale;
    let east_z1 = (row + 1) as f32 * scale;

    let top_corner_coord = grid_size * (grid_size - 1);
    let top_corner_x0 = (col as f32 + 2.0) * scale;
    let top_corner_y0 = top_corner.heightmap[top_corner_coord as usize] * scale;
    let top_corner_z0 = (row as f32 - 1.0) * scale;

    if row == 0 {
        triangles.push(Vec3 {
            x: north_x0,
            y: north_y0,
            z: north_z0,
        });
        triangles.push(Vec3 {
            x: x0,
            y: y0,
            z: z0,
        });
        triangles.push(Vec3 {
            x: north_x1,
            y: north_y1,
            z: north_z1,
        });

        triangles.push(Vec3 {
            x: north_x1,
            y: north_y1,
            z: north_z1,
        });
        triangles.push(Vec3 {
            x: x0,
            y: y0,
            z: z0,
        });
        triangles.push(Vec3 {
            x: x1,
            y: y1,
            z: z1,
        });
    }
    if col == grid_size - 2 {
        triangles.push(Vec3 {
            x: x1,
            y: y1,
            z: z1,
        });
        triangles.push(Vec3 {
            x: x3,
            y: y3,
            z: z3,
        });
        triangles.push(Vec3 {
            x: east_x0,
            y: east_y0,
            z: east_z0,
        });

        triangles.push(Vec3 {
            x: east_x0,
            y: east_y0,
            z: east_z0,
        });
        triangles.push(Vec3 {
            x: x3,
            y: y3,
            z: z3,
        });
        triangles.push(Vec3 {
            x: east_x1,
            y: east_y1,
            z: east_z1,
        });
    }

    if col == grid_size - 2 && row == 0 {
        triangles.push(Vec3 {
            x: north_x1,
            y: north_y1,
            z: north_z1,
        });
        triangles.push(Vec3 {
            x: x1,
            y: y1,
            z: z1,
        });
        triangles.push(Vec3 {
            x: top_corner_x0,
            y: top_corner_y0,
            z: top_corner_z0,
        });

        triangles.push(Vec3 {
            x: top_corner_x0,
            y: top_corner_y0,
            z: top_corner_z0,
        });
        triangles.push(Vec3 {
            x: x1,
            y: y1,
            z: z1,
        });
        triangles.push(Vec3 {
            x: east_x0,
            y: east_y0,
            z: east_z0,
        });
    }
}

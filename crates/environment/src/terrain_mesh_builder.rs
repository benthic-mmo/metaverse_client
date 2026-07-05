use crate::land::Land;
use benthic_protocol::render_data::RenderObject;
use glam::Vec3;
use std::collections::HashMap;
use uuid::Uuid;

struct MeshBuilder<'a> {
    layer: &'a Land,
    north_layer: &'a Land,
    east_layer: &'a Land,
    top_corner: &'a Land,

    scale: f32,
    grid_size: usize,

    vertices: Vec<Vec3>,
    indices: Vec<u16>,
    vertex_map: HashMap<(i32, i32, i32), u16>,
}

impl<'a> MeshBuilder<'a> {
    fn new(
        layer: &'a Land,
        north_layer: &'a Land,
        east_layer: &'a Land,
        top_corner: &'a Land,
        scale: f32,
    ) -> Self {
        let grid_size = layer.terrain_header.patch_size;

        Self {
            layer,
            north_layer,
            east_layer,
            top_corner,
            scale,
            grid_size,
            vertices: Vec::new(),
            indices: Vec::new(),
            vertex_map: HashMap::new(),
        }
    }

    fn add_vertex(&mut self, v: Vec3) -> u16 {
        let key = (
            (v.x * 1_000.0) as i32,
            (v.y * 1_000.0) as i32,
            (v.z * 1_000.0) as i32,
        );

        if let Some(&i) = self.vertex_map.get(&key) {
            i
        } else {
            let i = self.vertices.len() as u16;
            self.vertices.push(v);
            self.vertex_map.insert(key, i);
            i
        }
    }

    fn build(&mut self) {
        for row in 0..self.grid_size - 1 {
            for col in 0..self.grid_size - 1 {
                self.build_quad(row, col);
                self.stitch(row, col);
            }
        }
    }

    fn build_quad(&mut self, row: usize, col: usize) {
        let gs = self.grid_size;

        let top_left = row * gs + col;
        let top_right = top_left + 1;
        let bottom_left = (row + 1) * gs + col;
        let bottom_right = bottom_left + 1;

        let v0 = Vec3::new(col as f32, self.layer.heightmap[top_left], row as f32);
        let v1 = Vec3::new(
            (col + 1) as f32,
            self.layer.heightmap[top_right],
            row as f32,
        );
        let v2 = Vec3::new(
            col as f32,
            self.layer.heightmap[bottom_left],
            (row + 1) as f32,
        );
        let v3 = Vec3::new(
            (col + 1) as f32,
            self.layer.heightmap[bottom_right],
            (row + 1) as f32,
        );

        let i0 = self.add_vertex(v0);
        let i1 = self.add_vertex(v1);
        let i2 = self.add_vertex(v2);
        let i3 = self.add_vertex(v3);

        self.indices.extend_from_slice(&[i0, i2, i1]);
        self.indices.extend_from_slice(&[i1, i2, i3]);
    }

    fn stitch(&mut self, row: usize, col: usize) {
        let gs = self.grid_size;
        let scale = self.scale;

        let top_left = row * gs + col;
        let top_right = top_left + 1;
        let bottom_right = (row + 1) * gs + col + 1;

        let v0 = Vec3::new(
            col as f32,
            self.layer.heightmap[top_left] * scale,
            row as f32,
        );

        let v1 = Vec3::new(
            (col + 1) as f32,
            self.layer.heightmap[top_right] * scale,
            row as f32,
        );

        let v3 = Vec3::new(
            (col + 1) as f32,
            self.layer.heightmap[bottom_right] * scale,
            (row + 1) as f32,
        );

        // NORTH EDGE
        if row == 0 {
            let north_top_left = (gs - 1) * gs + col;
            let north_top_right = north_top_left + 1;

            let n0 = Vec3::new(
                col as f32,
                self.north_layer.heightmap[north_top_left] * scale,
                row as f32 - 1.0,
            );

            let n1 = Vec3::new(
                (col + 1) as f32,
                self.north_layer.heightmap[north_top_right] * scale,
                row as f32 - 1.0,
            );

            let i_n0 = self.add_vertex(n0);
            let i_n1 = self.add_vertex(n1);
            let i_v0 = self.add_vertex(v0);
            let i_v1 = self.add_vertex(v1);

            self.indices.extend_from_slice(&[i_n0, i_v0, i_n1]);
            self.indices.extend_from_slice(&[i_n1, i_v0, i_v1]);
        }

        // EAST EDGE
        if col == gs - 2 {
            let east_top = gs * row;
            let east_bottom = gs * (row + 1);

            let e0 = Vec3::new(
                (col as f32 + 2.0) * scale,
                self.east_layer.heightmap[east_top] * scale,
                row as f32,
            );

            let e1 = Vec3::new(
                (col as f32 + 2.0) * scale,
                self.east_layer.heightmap[east_bottom] * scale,
                (row + 1) as f32,
            );

            let i_v1 = self.add_vertex(v1);
            let i_v3 = self.add_vertex(v3);
            let i_e0 = self.add_vertex(e0);
            let i_e1 = self.add_vertex(e1);

            self.indices.extend_from_slice(&[i_v1, i_v3, i_e0]);
            self.indices.extend_from_slice(&[i_e0, i_v3, i_e1]);
        }

        // CORNER EDGE
        if col == gs - 2 && row == 0 {
            let corner_index = gs * (gs - 1);

            let c0 = Vec3::new(
                (col as f32 + 2.0) * scale,
                self.top_corner.heightmap[corner_index] * scale,
                row as f32 - 1.0,
            );

            let i_n1 = self.add_vertex(v1);
            let i_v1 = self.add_vertex(v1);
            let i_e0 = self.add_vertex(v3);
            let i_c0 = self.add_vertex(c0);

            self.indices.extend_from_slice(&[i_n1, i_v1, i_c0]);
            self.indices.extend_from_slice(&[i_c0, i_v1, i_e0]);
        }
    }
}

/// Generates a RenderObject for the terrain
pub fn build_terrain(
    layer: &Land,
    north_layer: &Land,
    east_layer: &Land,
    top_corner: &Land,
) -> RenderObject {
    let mut builder = MeshBuilder::new(layer, north_layer, east_layer, top_corner, 1.0);

    builder.build();

    RenderObject {
        vertices: builder.vertices,
        indices: builder.indices,
        id: Uuid::nil(),
        name: layer.terrain_header.location.to_string(),
        skin: None,
        texture: None,
        uv: None,
    }
}

use benthic_protocol::{
    render_data::{RenderObject, SkinData},
    skeleton::Skeleton,
};
use glam::{Vec3, Vec4};
use log::warn;
use metaverse_agent::skeleton::create_skeleton;
use metaverse_messages::http::mesh::Mesh;
use std::path::Path;
use uuid::Uuid;

pub fn create_render_object(
    mesh: Mesh,
    name: String,
    texture_path: &Path,
    asset_id: Uuid,
) -> Result<RenderObject, std::io::Error> {
    let domain = &mesh.high_level_of_detail.texture_coordinate_domain;
    let uvs: Vec<[f32; 2]> = mesh
        .high_level_of_detail
        .texture_coordinate
        .iter()
        .map(|tc| {
            // Normalize U and V from 0..65535 to 0..1
            let u_norm = tc.u as f32 / 65535.0;
            let v_norm = tc.v as f32 / 65535.0;

            // Flip V axis
            let v_flipped = 1.0 - v_norm;

            [
                domain.min[0] + u_norm * (domain.max[0] - domain.min[0]),
                domain.min[1] + v_flipped * (domain.max[1] - domain.min[1]),
            ]
        })
        .collect();

    let object = if let Some(skin) = &mesh.skin {
        // Apply bind shape matrix
        let vertices: Vec<Vec3> = mesh
            .high_level_of_detail
            .vertices
            .iter()
            .map(|v| {
                let v4 = skin.bind_shape_matrix * Vec4::new(v.x, v.y, v.z, 1.0);
                Vec3::new(v4.x, v4.y, v4.z)
            })
            .collect();

        let skeleton = create_skeleton(name.clone(), asset_id, skin).unwrap_or_else(|e| {
            warn!("Failed to create skeleton: {:?}", e);
            Skeleton::default()
        });

        let skin_data = SkinData {
            skeleton,
            weights: mesh
                .high_level_of_detail
                .weights
                .clone()
                .unwrap_or_default(),
            joint_names: skin.joint_names.clone(),
            inverse_bind_matrices: skin.inverse_bind_matrices.clone(),
        };

        RenderObject {
            name,
            id: asset_id,
            indices: mesh.high_level_of_detail.indices,
            vertices,
            skin: Some(skin_data),
            texture: Some(texture_path.to_path_buf()),
            uv: Some(uvs),
        }
    } else {
        let vertices: Vec<Vec3> = mesh.high_level_of_detail.vertices;
        //.iter()
        //.map(|v| apply_scale_rotation(*v, scale, rotation))
        //.collect();

        RenderObject {
            name,
            id: asset_id,
            indices: mesh.high_level_of_detail.indices,
            vertices,
            skin: None,
            texture: Some(texture_path.to_path_buf()),
            uv: Some(uvs),
        }
    };
    Ok(object)
}

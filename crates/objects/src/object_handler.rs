use benthic_default_asset_converter::default_texture_path;
use benthic_protocol::{
    objects::GeneratorObject,
    render_data::{RenderObject, SkinData},
    session::{CacheDir, create_sub_object_dir, write_json},
    skeleton::{Skeleton, SkinJoint},
};
use glam::Vec3;
use log::{info, warn};
use metaverse_avatar::skeleton::create_skeleton;
use metaverse_mesh::mesh::generate::generate_object_mesh;
use metaverse_messages::{http::mesh::Mesh, utils::object_types::ObjectType};
use metaverse_store::initialize_sqlite::Cache;
use std::path::{Path, PathBuf};
use uuid::Uuid;

use crate::{
    errors::{DownloadError, MeshBuildError},
    http_handler::{download_asset, download_texture},
    object_updates::{
        DownloadMeshObjectData, GenerateMeshData, ObjectUpdateAction, RenderObjectData,
    },
};

pub async fn handle_texture(
    base_dir: PathBuf,
    texture_id: Uuid,
    server_endpoint: String,
) -> Result<PathBuf, DownloadError> {
    let texture_path = base_dir.join(format!("{:?}.png", texture_id));
    download_texture(
        ObjectType::Texture.to_string(),
        texture_id,
        &server_endpoint,
        &texture_path,
    )
    .await?;
    Ok(texture_path)
}

pub async fn download_mesh_object(
    cache: Cache,
    server_endpoint: String,
    data: &DownloadMeshObjectData,
    out_dir: PathBuf,
) -> Result<ObjectUpdateAction, DownloadError> {
    let base_dir = create_sub_object_dir(&out_dir, &data.object.sculpt_id.to_string())?;

    let mesh = Mesh::from_bytes(
        &download_asset(
            ObjectType::Mesh.to_string(),
            data.object.sculpt_id,
            &server_endpoint,
        )
        .await?,
    )?;

    let texture_path = if let Some(ref texture) = data.object.texture {
        handle_texture(base_dir.clone(), texture.texture_id, server_endpoint).await?
    } else {
        default_texture_path()
    };

    let render_object = create_render_object(
        mesh,
        "name".to_string(),
        &texture_path,
        data.object.sculpt_id,
    )?;

    let json_path = write_json(
        &render_object,
        &data.object.sculpt_id.to_string(),
        CacheDir::Object(data.object.sculpt_id),
        &out_dir,
    )?;

    cache
        .object
        .update_json_path(
            data.object.full_id,
            data.object.sculpt_id,
            &json_path.to_string_lossy(),
        )
        .await?;

    Ok(ObjectUpdateAction::GenerateFromJSON(GenerateMeshData {
        object: GeneratorObject {
            full_id: data.object.full_id,
            local_id: data.object.local_id,
            parent_id: data.object.parent,
            rotation: data.object.rotation,
            scale: data.object.scale,
            position: data.object.position,
        },
        asset_id: data.object.sculpt_id,
        base_dir,
        json_path,
    }))
}

pub async fn mesh_from_json(
    cache: Cache,
    data: GenerateMeshData,
) -> Result<ObjectUpdateAction, MeshBuildError> {
    let glb_path = data.base_dir.join(format!("{:?}_high.glb", data.asset_id));
    generate_object_mesh(data.json_path, glb_path.clone())?;
    info!("Rendering object {:?}", data.asset_id);
    cache
        .object
        .update_glb_path(data.object.full_id, &glb_path.to_string_lossy())
        .await?;
    Ok(ObjectUpdateAction::Render(RenderObjectData {
        mesh_path: Some(glb_path),
        base_dir: data.base_dir,
        asset_id: data.asset_id,
        object: data.object,
        retry_count: 0,
        download: None,
    }))
}

pub fn create_render_object(
    mesh: Mesh,
    name: String,
    texture_path: &Path,
    asset_id: Uuid,
) -> Result<RenderObject, DownloadError> {
    let domain = &mesh.high_level_of_detail.texture_coordinate_domain;
    let uvs: Vec<[f32; 2]> = mesh
        .high_level_of_detail
        .texture_coordinate
        .iter()
        .map(|tc| {
            // Normalize U and V from 0..65535 to 0..1
            let u_norm = tc.u as f32 / 65535.0;
            let v_norm = tc.v as f32 / 65535.0;
            [
                domain.min[0] + u_norm * (domain.max[0] - domain.min[0]),
                // we need to flip this to make these render correctly in GLTF.
                1.0 - (domain.min[1] + v_norm * (domain.max[1] - domain.min[1])),
            ]
        })
        .collect();

    let object = if let Some(skin) = &mesh.skin {
        // Apply bind shape matrix
        let vertices = mesh
            .high_level_of_detail
            .vertices
            .iter()
            .map(|v| skin.bind_shape_matrix.transform_point3(*v))
            .collect::<Vec<_>>();

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
            joint_names: skin
                .joints
                .iter()
                .filter_map(|joint| match joint {
                    SkinJoint::Joint(name) => Some(*name),
                    SkinJoint::Collision(_) => None,
                    _ => None,
                })
                .collect(),
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
            normals: mesh.high_level_of_detail.normals,
        }
    } else {
        let vertices: Vec<Vec3> = mesh.high_level_of_detail.vertices;
        RenderObject {
            name,
            id: asset_id,
            indices: mesh.high_level_of_detail.indices,
            vertices,
            skin: None,
            texture: Some(texture_path.to_path_buf()),
            uv: Some(uvs),
            normals: mesh.high_level_of_detail.normals,
        }
    };
    Ok(object)
}

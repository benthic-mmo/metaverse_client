use benthic_protocol::{
    objects::GeneratorObject,
    render_data::{RenderObject, SkinData},
    session::{CacheDir, create_sub_object_dir, write_json},
    skeleton::Skeleton,
};
use default_asset_converter::default_texture_path;
use glam::{Vec3, Vec4};
use log::{info, warn};
use metaverse_avatar::skeleton::create_skeleton;
use metaverse_cache::object_update::{
    sqlite_update_object_glb_path, sqlite_update_object_json_path,
};
use metaverse_mesh::mesh::generate::generate_object_mesh;
use metaverse_messages::{http::mesh::Mesh, utils::object_types::ObjectType};
use sqlx::{Pool, Sqlite};
use std::path::{Path, PathBuf};
use uuid::Uuid;

use crate::{
    errors::{DownloadError, MeshBuildError},
    http_handler::{download_mesh, download_texture},
    object_updates::{DownloadObjectData, GenerateMeshData, ObjectUpdateAction, RenderObjectData},
};

pub async fn handle_texture(
    base_dir: PathBuf,
    texture_id: Uuid,
    server_endpoint: String,
) -> PathBuf {
    let texture_path = base_dir.join(format!("{:?}.png", texture_id));
    match download_texture(
        ObjectType::Texture.to_string(),
        texture_id,
        &server_endpoint,
        &texture_path,
    )
    .await
    {
        Ok(_) => texture_path,
        Err(e) => {
            warn!(
                "Failed to download prim texture: {:?}, {:?}. Using default texture {:}",
                e,
                texture_id,
                default_texture_path().to_string_lossy()
            );
            default_texture_path()
        }
    }
}

pub async fn download_object(
    db_conn: &Pool<Sqlite>,
    server_endpoint: String,
    data: DownloadObjectData,
) -> Result<ObjectUpdateAction, DownloadError> {
    let base_dir = create_sub_object_dir(&data.asset_id.to_string())?;

    let mesh = download_mesh(
        ObjectType::Mesh.to_string(),
        data.asset_id,
        &server_endpoint,
    )
    .await?;

    let texture_path = handle_texture(base_dir.clone(), data.texture_id, server_endpoint).await;

    let render_object =
        create_render_object(mesh, "name".to_string(), &texture_path, data.asset_id)?;

    let json_path = write_json(
        &render_object,
        &data.asset_id.to_string(),
        CacheDir::Object(data.asset_id),
    )?;

    sqlite_update_object_json_path(
        db_conn,
        data.object.full_id,
        data.asset_id,
        &json_path.to_string_lossy(),
    )
    .await?;

    Ok(ObjectUpdateAction::GenerateFromJSON(GenerateMeshData {
        object: GeneratorObject {
            full_id: data.object.full_id,
            local_id: data.object.local_id,
            parent_id: data.object.parent_id,
            rotation: data.object.rotation,
            scale: data.object.scale,
            position: data.object.position,
        },
        asset_id: data.asset_id,
        base_dir,
        json_path,
    }))
}

pub async fn mesh_from_json(
    db_conn: &Pool<Sqlite>,
    data: GenerateMeshData,
) -> Result<ObjectUpdateAction, MeshBuildError> {
    let glb_path = data.base_dir.join(format!("{:?}_high.glb", data.asset_id));
    generate_object_mesh(data.json_path, glb_path.clone())?;
    info!("Rendering object {:?}", data.asset_id);
    sqlite_update_object_glb_path(db_conn, data.object.full_id, &glb_path.to_string_lossy())
        .await?;
    Ok(ObjectUpdateAction::Render(RenderObjectData {
        mesh_path: glb_path,
        base_dir: data.base_dir,
        asset_id: data.asset_id,
        object: data.object,
        retry_count: 0,
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

use benthic_protocol::{
    objects::GeneratorObject,
    render_data::{RenderFace, RenderObject, SkinData},
    session::{CacheDir, create_sub_object_dir, write_json},
    skeleton::{Skeleton, SkinJoint},
};
use log::{info, warn};
use metaverse_avatar::skeleton::create_skeleton;
use metaverse_mesh::mesh::generate::generate_object_mesh;
use metaverse_messages::{http::mesh::Mesh, utils::object_types::ObjectType};
use metaverse_store::initialize_sqlite::Cache;
use std::{collections::HashMap, path::PathBuf};
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

    let mut texture_paths = HashMap::new();
    if let Some(texture) = &data.object.texture {
        let texture_path = handle_texture(
            base_dir.to_path_buf(),
            texture.default.texture_id,
            server_endpoint.clone(),
        )
        .await?;
        texture_paths.insert(u32::MAX, texture_path);
        for (face_index, face) in &texture.faces {
            let texture_path = handle_texture(
                base_dir.to_path_buf(),
                face.texture_id,
                server_endpoint.clone(),
            )
            .await?;

            texture_paths.insert(*face_index, texture_path);
        }
    }

    let render_object = create_render_object(
        mesh,
        "name".to_string(),
        texture_paths,
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
    texture_paths: HashMap<u32, PathBuf>,
    asset_id: Uuid,
) -> Result<RenderObject, DownloadError> {
    let skin = mesh.skin.as_ref();

    let faces = mesh
        .high_level_of_detail
        .iter()
        .enumerate()
        .filter(|(_, face)| !face.no_geometry)
        .map(|(face_index, face)| {
            let domain = &face.texture_coordinate_domain;

            let uv = face
                .texture_coordinate
                .iter()
                .map(|tc| {
                    let u = tc.u as f32 / 65535.0;
                    let v = tc.v as f32 / 65535.0;

                    [
                        domain.min[0] + u * (domain.max[0] - domain.min[0]),
                        1.0 - (domain.min[1] + v * (domain.max[1] - domain.min[1])),
                    ]
                })
                .collect();

            let vertices = face
                .vertices
                .iter()
                .map(|vertex| {
                    skin.map_or(*vertex, |skin| {
                        skin.bind_shape_matrix.transform_point3(*vertex)
                    })
                })
                .collect();

            RenderFace {
                face_index: face_index as u32,
                vertices,
                indices: face.indices.clone(),
                uv,
                normals: face.normals.clone(),
                texture: texture_paths
                    .get(&(face_index as u32))
                    .or_else(|| texture_paths.get(&u32::MAX))
                    .cloned(),
                weights: face.weights.clone(),
            }
        })
        .collect();

    let skin_data = if let Some(skin) = skin {
        let skeleton = create_skeleton(name.clone(), asset_id, skin).unwrap_or_else(|e| {
            warn!("Failed to create skeleton: {:?}", e);
            Skeleton::default()
        });

        Some(SkinData {
            skeleton,
            weights: mesh
                .high_level_of_detail
                .iter()
                .filter(|face| !face.no_geometry)
                .flat_map(|face| face.weights.clone().unwrap_or_default())
                .collect(),
            joint_names: skin
                .joints
                .iter()
                .filter_map(|joint| match joint {
                    SkinJoint::Joint(name) => Some(*name),
                    _ => None,
                })
                .collect(),
            inverse_bind_matrices: skin.inverse_bind_matrices.clone(),
        })
    } else {
        None
    };

    Ok(RenderObject {
        name,
        id: asset_id,
        faces,
        skin: skin_data,
    })
}

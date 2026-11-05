use benthic_protocol::session::storage::{CacheDir, create_sub_object_dir, write_json};
use bytes::Bytes;
use metaverse_mesh::mesh::generate::generate_object_mesh;
use metaverse_messages::http::mesh::Mesh;
use metaverse_store::initialize_sqlite::Cache;
use std::{collections::HashMap, path::PathBuf};

use crate::{
    errors::MeshBuildError,
    object_updates::{
        AvatarObjectData, MeshObjectData, ObjectUpdateAction, RenderMeshData, RenderObjectJSONData,
    },
    utils::create_mesh_render_object,
};

/// Handles processing a mesh object update into a renderable object.
///
/// Returns an [`ObjectUpdateAction::Render`] on success
///
/// # Parameters
/// - `cache`: The Session's cache
/// - `data`: The mesh object data required for rendering
/// - `out_dir`: The base directory to write mesh data to
/// - `mesh_bytes`: mesh object's raw bytes
/// - `texture_paths`: Mapping of a u32 face to the path of the texture on that face
///
/// # Errors
/// - Returns a [`MeshBuildError`] on failure
///
/// # Side Effects
/// [Storage: Filesystem + SQLite]
pub async fn handle_mesh_object(
    cache: Cache,
    data: &MeshObjectData,
    base_dir: PathBuf,
    glb_path: PathBuf,
    mesh_bytes: Bytes,
    texture_paths: HashMap<u32, PathBuf>,
    material_paths: HashMap<u32, PathBuf>,
) -> Result<ObjectUpdateAction, MeshBuildError> {
    let mesh = Mesh::from_bytes(&mesh_bytes)?;

    let render_object = create_mesh_render_object(
        mesh,
        "name".to_string(),
        texture_paths,
        material_paths,
        data.sculpt_id,
        &data.texture,
        &data.avatar_attachment,
        data.texture_animation.clone(),
    )?;

    // cache render object on disk
    let json_path = base_dir.join(format!("{}.json", data.sculpt_id));
    let json = serde_json::to_vec(&render_object)?;
    tokio::fs::write(&json_path, &json).await?;

    // generate mesh from the glb
    generate_object_mesh(&json_path, &glb_path)?;

    // update the generated render object in the db
    cache
        .object
        .update_json_path(
            data.object_info.full_id,
            data.sculpt_id,
            &json_path.to_string_lossy(),
        )
        .await?;

    // cache the generated mesh's data in sqlite
    cache
        .object
        .update_glb_path(data.object_info.full_id, &glb_path.to_string_lossy())
        .await?;

    // return a render mesh data action from the generated mesh
    Ok(ObjectUpdateAction::Render(RenderMeshData {
        mesh_path: Some(glb_path),
        base_dir,
        asset_id: data.sculpt_id,
        object: data.object_info.clone(),
        retry_count: 0,
        download: None,
    }))
}

pub async fn handle_mesh_avatar_object(
    cache: Cache,
    data: &MeshObjectData,
    out_dir: PathBuf,
    mesh_bytes: Bytes,
    texture_paths: HashMap<u32, PathBuf>,
    material_paths: HashMap<u32, PathBuf>,
) -> Result<ObjectUpdateAction, MeshBuildError> {
    let Some(attachment) = data.avatar_attachment.clone() else {
        return Err(MeshBuildError::EmptyAvatarData);
    };

    let mesh = Mesh::from_bytes(&mesh_bytes)?;

    let render_object = create_mesh_render_object(
        mesh,
        "name".to_string(),
        texture_paths,
        material_paths,
        data.sculpt_id,
        &data.texture,
        &data.avatar_attachment,
        data.texture_animation.clone(),
    )?;

    // cahche generated render object on the filesystem
    let json_path = write_json(
        &render_object,
        &data.sculpt_id.to_string(),
        CacheDir::Agent(attachment.agent_id),
        &out_dir,
    )?;

    // Include the skeleton in the update data, to be handled on the add object without having to
    // reread the file.
    let skeleton = if let Some(skin) = render_object.skin {
        Some(skin.skeleton)
    } else {
        None
    };

    // update the generated render object in the sqlite db
    cache
        .object
        .update_json_path(
            data.object_info.full_id,
            data.sculpt_id,
            &json_path.to_string_lossy(),
        )
        .await?;

    // return a render mesh data action from the generated mesh
    Ok(ObjectUpdateAction::AvatarObject(AvatarObjectData {
        json_path,
        asset_id: data.sculpt_id,
        agent_id: attachment.agent_id,
        object_info: data.object_info.clone(),
        skeleton_update: skeleton,
    }))
}

/// Handles processing a render object json into a mesh object
///
/// Returns an [`ObjectUpdateAction::Render`] on success
/// TODO: this may not actually be smart to have around. consider removing this.
///
/// # Parameters
/// - `cache`: Session's cache
/// - `data`: Data about the render object, such as its file path, ID and position in-world
///
/// #Errors
/// - returns a [`MeshBuildError`] on failure
///
/// # Side Effects
/// [Storage: Filesystem + SQLite]
pub async fn mesh_from_json(
    cache: Cache,
    data: RenderObjectJSONData,
) -> Result<ObjectUpdateAction, MeshBuildError> {
    let glb_path = data.base_dir.join(format!("{:?}_high.glb", data.asset_id));
    generate_object_mesh(&data.json_path, &glb_path)?;
    cache
        .object
        .update_glb_path(data.object.full_id, &glb_path.to_string_lossy())
        .await?;
    Ok(ObjectUpdateAction::Render(RenderMeshData {
        mesh_path: Some(glb_path),
        base_dir: data.base_dir,
        asset_id: data.asset_id,
        object: data.object,
        retry_count: 0,
        download: None,
    }))
}

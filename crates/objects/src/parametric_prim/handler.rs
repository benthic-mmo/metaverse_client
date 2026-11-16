use super::{faces, path, profile};
use crate::{
    errors::ObjectUpdateError,
    object_updates::{ObjectUpdateAction, ParametricPrimData, RenderMeshData},
};
use benthic_protocol::{
    objects::types::{RenderObject, TextureAnimation},
    session::storage::{BenthicDirs, cache_enabled},
};
use metaverse_mesh::mesh::generate::generate_object_mesh;
use metaverse_messages::utils::texture_entry::TextureEntries;
use std::{
    collections::HashMap,
    hash::{Hash, Hasher},
    path::PathBuf,
};
use uuid::Uuid;

pub(crate) struct TextureContext<'a> {
    pub(crate) texture_paths: &'a HashMap<u32, PathBuf>,
    pub(crate) texture: &'a Option<TextureEntries>,
    pub(crate) texture_animation: &'a Option<TextureAnimation>,
    pub(crate) material_paths: &'a HashMap<u32, PathBuf>,
}

const MESH_GENERATOR_VERSION: &str = env!("CARGO_PKG_VERSION");

/// Creates a parametric prim from parametric prim data taken from an object update.
///
/// # Parameters
/// - `dirs`: The benthic directory manager
/// - `object`: Data required for building a parametric prim
/// - `texture_paths`: a HashMap keyed to the face with a path to each texture on disk
///
/// # Errors
/// Returns an [`ObjectUpdateError`] if generation fails
///
/// # Side Effects
/// [Storage: Filesystem]
pub async fn handle_parametric_prim_object(
    dirs: &BenthicDirs,
    object: &ParametricPrimData,
    texture_paths: HashMap<u32, PathBuf>,
    material_paths: HashMap<u32, PathBuf>,
    asset_id: Uuid,
) -> Result<Vec<ObjectUpdateAction>, ObjectUpdateError> {
    let cache_key = create_cache_key(object);
    let base_dir = dirs.ensure_prim_dir(cache_key)?;
    let json_path = dirs.prim_json(cache_key);
    let gltf_path = dirs.prim_gltf(cache_key);

    if !gltf_path.exists() || !cache_enabled() {
        if !json_path.exists() || !cache_enabled() {
            let render_object =
                build_render_object(object, &texture_paths, &material_paths, asset_id)?;
            std::fs::write(&json_path, serde_json::to_vec_pretty(&render_object)?)?;
        }
        generate_object_mesh(&json_path, &gltf_path)?;
    }

    Ok(vec![ObjectUpdateAction::Render(RenderMeshData {
        mesh_path: Some(gltf_path),
        base_dir,
        asset_id,
        object: object.object_info.clone(),
        download: None,
        retry_count: 0,
    })])
}

// TODO: IMPLEMENT THIS
pub async fn handle_parametric_prim_avatar_object() {}

fn build_render_object(
    object: &ParametricPrimData,
    texture_paths: &HashMap<u32, PathBuf>,
    material_paths: &HashMap<u32, PathBuf>,
    asset_id: Uuid,
) -> Result<RenderObject, ObjectUpdateError> {
    let profile = profile::build_profile(
        object.path_data.hollow_shape,
        object.path_data.profile_hollow,
        object.path_data.profile_shape,
    )?;

    let layers = path::follow_path(object.path_data.clone(), &profile)?;

    let has_hollow = object.path_data.profile_hollow > 0.0;
    let faces = faces::split_faces(
        &layers,
        &object.path_data,
        has_hollow,
        &TextureContext {
            texture_paths,
            texture: &object.texture,
            texture_animation: &object.texture_animation,
            material_paths,
        },
    )?;

    Ok(RenderObject {
        name: "Prim".to_owned(),
        id: asset_id,
        attachment_data: None,
        faces,
        skin: None,
    })
}

fn create_cache_key(object: &ParametricPrimData) -> u64 {
    let mut hasher = std::collections::hash_map::DefaultHasher::new();

    object.path_data.hash(&mut hasher);
    MESH_GENERATOR_VERSION.hash(&mut hasher);

    if let Some(texture) = &object.texture {
        texture.default.texture_id.hash(&mut hasher);
        hash_color(texture.default.rgba, &mut hasher);

        let mut faces: Vec<_> = texture.faces.iter().collect();
        faces.sort_by_key(|(index, _)| **index);

        for (index, face) in faces {
            index.hash(&mut hasher);
            face.texture_id.hash(&mut hasher);
            hash_color(face.rgba, &mut hasher);
        }
    }

    hasher.finish()
}

fn hash_color<H: Hasher>(color: rgb::Rgba<u8>, hasher: &mut H) {
    (color.r, color.g, color.b, color.a).hash(hasher);
}

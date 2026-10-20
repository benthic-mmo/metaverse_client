use super::{faces, path, profile};
use crate::{
    errors::ObjectUpdateError,
    object_updates::{ObjectUpdateAction, RenderMeshData},
};
use benthic_protocol::{
    objects::{objects::ParametricPrimData, types::RenderObject},
    session::session::{cache_enabled, create_sub_object_dir},
};
use metaverse_mesh::mesh::generate::generate_object_mesh;
use metaverse_messages::utils::{path::PrimPath, texture_entry::TextureEntries};
use std::{
    collections::HashMap,
    hash::{Hash, Hasher},
    path::{Path, PathBuf},
};
use uuid::Uuid;

const MESH_GENERATOR_VERSION: &str = env!("CARGO_PKG_VERSION");

/// Creates a parametric prim from parametric prim data taken from an object update.
///
/// This fully converts raw object update data to a mesh file that can be sent to the UI.
///
/// # Parameters
/// - `out_dir`: The base dir for the mesh's GLTF and JSON data
/// - `object`: Data required for building a parametric prim
/// - `texture_paths`: a HashMap keyed to the face with a path to each texture on disk
///
/// # Errors
/// Returns an [`ObjectUpdateError`] if generation fails
///
/// # Side Effects
/// [Storage: Filesystem]
pub async fn handle_parametric_prim(
    out_dir: &Path,
    object: &ParametricPrimData<TextureEntries, PrimPath>,
    texture_paths: HashMap<u32, PathBuf>,
    asset_id: Uuid,
) -> Result<Vec<ObjectUpdateAction>, ObjectUpdateError> {
    let cache_key = create_cache_key(object);
    let base_dir = create_sub_object_dir(out_dir, &format!("{cache_key:016x}"))?;

    let json_path = base_dir.join(format!("{cache_key}.json"));
    let glb_path = base_dir.join(format!("{cache_key}.glb"));

    // if the glb path doesn't exist or we're running without cache,
    // check to see if the json path exists. if it doesn't, then build the render object
    if !glb_path.exists() || !cache_enabled() {
        if !json_path.exists() || !cache_enabled() {
            let render_object = build_render_object(object, &texture_paths, asset_id)?;
            std::fs::write(&json_path, serde_json::to_vec_pretty(&render_object)?)?;
        }
        generate_object_mesh(&json_path, &glb_path)?;
    }

    Ok(vec![ObjectUpdateAction::Render(RenderMeshData {
        mesh_path: Some(glb_path),
        base_dir,
        asset_id,
        object: object.object_info.clone(),
        download: None,
        retry_count: 0,
    })])
}

fn build_render_object(
    object: &ParametricPrimData<TextureEntries, PrimPath>,
    texture_paths: &HashMap<u32, PathBuf>,
    asset_id: Uuid,
) -> Result<RenderObject, ObjectUpdateError> {
    let profile = profile::build_profile(
        object.path_data.hollow_shape,
        object.path_data.profile_hollow,
        object.path_data.profile_shape,
    )?;

    let layers = path::follow_path(object.path_data.clone(), &profile)?;

    let faces = faces::split_faces(
        &layers,
        object.path_data.profile_shape,
        object.path_data.hollow_shape,
        object.path_data.profile_hollow > 0.0,
        texture_paths,
        &object.texture,
        object.texture_animation.clone(),
    )?;

    Ok(RenderObject {
        name: "Prim".to_owned(),
        id: asset_id,
        attachment_data: None,
        faces,
        skin: None,
    })
}

fn create_cache_key(object: &ParametricPrimData<TextureEntries, PrimPath>) -> u64 {
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

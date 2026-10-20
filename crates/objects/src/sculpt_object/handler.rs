use super::{finalize::finalize_sculpt_object, geometry::build_sculpt_geometry};
use crate::{
    errors::ObjectUpdateError,
    object_updates::{ObjectUpdateAction, RenderMeshData},
};
use benthic_protocol::{
    objects::objects::SculptObjectData,
    session::session::{cache_enabled, create_sub_object_dir},
};
use metaverse_mesh::mesh::generate::generate_object_mesh;
use metaverse_messages::{
    udp::object::object_update::SculptData, utils::texture_entry::TextureEntries,
};
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};
use uuid::Uuid;

/// Creates a sculpt object from the sculpt data taken from an object update
///
/// This fully converts raw sculpt object data to a mesh file that can be sent to the UI.
///
/// # Parameters
/// - `out_dir`: The base dir for the mesh's GLTF and JSON data
/// - `texture_path`: the path on disk to the sculpt object's sculpt texture
/// - `object`: Data required for building a sculpt object
/// - `texture_paths`: a HashMap keyed to the face with a path to each texture on disk
///
/// # Errors
/// Returns an [`ObjectUpdateError`] if generation fails
///
/// # Side Effects
/// [Storage: Filesystem]
pub async fn handle_sculpt_object(
    out_dir: &Path,
    texture_path: &Path,
    object: &SculptObjectData<TextureEntries, SculptData>,
    texture_paths: HashMap<u32, PathBuf>,
    asset_id: Uuid,
) -> Result<Vec<ObjectUpdateAction>, ObjectUpdateError> {
    let base_dir = create_sub_object_dir(out_dir, &object.data.texture_id.to_string())?;

    let file_path = base_dir.join(format!("{}.json", object.data.texture_id));
    let glb_path = base_dir.join(format!("{}.glb", object.data.texture_id));

    if !glb_path.exists() || !cache_enabled() {
        if !file_path.exists() || !cache_enabled() {
            let image = image::open(texture_path)?.into_rgb8();

            let geometry = build_sculpt_geometry(
                &image,
                &object.data.sculpt_type,
                object.data.mirror,
                object.data.invert,
            )?;

            let render_object = finalize_sculpt_object(object, &texture_paths, geometry, asset_id);

            std::fs::write(&file_path, serde_json::to_vec_pretty(&render_object)?)?;
        }

        generate_object_mesh(&file_path, &glb_path)?;
    }

    Ok(vec![ObjectUpdateAction::Render(RenderMeshData {
        mesh_path: Some(glb_path),
        asset_id,
        base_dir,
        object: object.object_info.clone(),
        download: None,
        retry_count: 0,
    })])
}

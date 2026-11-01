use crate::errors::ObjectUpdateError;
use benthic_protocol::{
    objects::{
        types::TextureAnimation,
        types::{AttachmentObjectData, ObjectInstanceInfo},
    },
    session::{storage::create_sub_object_dir, types::DownloadState},
};
use log::warn;
use metaverse_messages::{
    udp::object::{
        object_update::{AttachItem, MaterialsData, SculptData},
        object_update_cached::CachedObjectData,
        request_multiple_objects::CacheMissType,
    },
    utils::{object_types::ObjectType, path::PrimPath, texture_entry::TextureEntries},
};
use metaverse_store::initialize_sqlite::Cache;
use std::path::PathBuf;
use tokio::sync::watch::{self};
use uuid::Uuid;

/// Data required to render a mesh for an instance of an object
///
/// RenderMeshData objects are special in that they can wait until the mesh is finished downloading
/// before rendering.
/// This is different from parametric prims and sculpts, since mesh downloads can take much longer
/// than texture downloads, and parametric prims come on the wire with all the information required.
#[derive(Debug)]
pub struct RenderMeshData {
    /// Common metadata containing parent ID, transforms, etc.
    pub object: ObjectInstanceInfo,
    /// path to the actual mesh file on disk
    pub mesh_path: Option<PathBuf>,
    /// directory containing the mesh file data
    pub base_dir: PathBuf,
    /// UUID of the mesh asset
    pub asset_id: Uuid,
    /// Number of downlod attempts made so far
    pub retry_count: u32,
    /// Receiver for the asset's download state, if it is in progress.
    pub download: Option<Vec<watch::Receiver<DownloadState>>>,
}

/// Information required to render an instance of an object from its RenderObject JSON on disk.
/// TODO: this may be removed in later versions.
#[derive(Debug)]
pub struct RenderObjectJSONData {
    /// Common metadata containing parent ID, transforms, etc.
    pub object: ObjectInstanceInfo,
    /// JSON path to the render object
    pub json_path: PathBuf,
    /// Directory containing the render object data
    pub base_dir: PathBuf,
    /// UUID of the render object
    pub asset_id: Uuid,
}

/// Information required to render a mesh object.
#[derive(Clone, Debug)]
pub struct MeshObjectData {
    /// Common metadata containing parent ID, transforms, etc.
    pub object_info: ObjectInstanceInfo,
    /// Type required to retrieve full data from the capability endpoint.
    pub object_type: ObjectType,
    /// Object's texture data.
    pub texture: Option<TextureEntries>,
    /// ID of the mesh/sculpt asset.
    pub sculpt_id: Uuid,
    /// CRC to enable cache invalidation.
    pub crc: u32,
    /// Region ID where the mesh belongs
    pub region_id: String,
    /// Information about texture animations
    pub texture_animation: Option<TextureAnimation>,
    /// Number of times this object has failed to download
    pub retry_count: u32,

    pub pbr_materials: Option<MaterialsData>,
}

/// Data object for rendering parametric prims
#[derive(Clone, Debug)]
pub struct ParametricPrimData {
    /// Common values for object's instance
    pub object_info: ObjectInstanceInfo,
    /// per-face texture data
    pub texture: Option<TextureEntries>,
    /// Texture animation data
    pub texture_animation: Option<TextureAnimation>,
    /// Path data for drawing the parametric prim's path
    pub path_data: PrimPath,
    /// How many times this object has been retried
    pub retry_count: u32,

    pub pbr_materials: Option<MaterialsData>,
}

/// Data object for rendering sculpt objects
#[derive(Debug, Clone)]
pub struct SculptObjectData {
    /// Common values for the object's instance
    pub object_info: ObjectInstanceInfo,
    /// data for building the sculpt object
    pub data: SculptData,
    /// per-face texture data
    pub texture: Option<TextureEntries>,
    /// texture animation data
    pub texture_animation: Option<TextureAnimation>,
    /// crc of the sculpt for cache invalidation
    pub crc: u32,
    /// region id where the sculpt was downloaded
    pub region_id: String,
    /// how many times this object has been retried
    pub retry_count: u32,

    pub pbr_materials: Option<MaterialsData>,
}

/// Actions that can result from handling an object update packet
pub enum ObjectUpdateAction {
    /// Download a mesh object and all of its associated textures and materials
    DownloadMesh(MeshObjectData),
    /// Render an object directly using its on-disk gltf representation
    Render(RenderMeshData),
    /// Generate mesh data from cached render object json
    /// TODO: this may be removed later
    GenerateFromRenderObject(RenderObjectJSONData),
    /// Handle an object not in the cache
    HandleCacheMiss((CacheMissType, u32)),
}

/// Handles object update cached.
///
/// On success, this returns a
/// [`ObjectUpdateAction::Render`] if the object is already cached, an
/// [`ObjectUpdateAction::GenerateFromRenderObject`] if only the render object JSON is cached
/// [`ObjectUpdateAction::HandleCacheMiss`] if the object is not present in the cache.
///
/// # Parameters
/// - `objects`: Parsed objects to check the cache for
/// - `cache`: Session's cache data that contains a connection with the sqlite db
/// - `region_id`: ID of the user's current region
/// - `out_dir`: object's root dir for storing generated gltf and json.
///
/// # Errors
/// - Returns [`ObjectUpdateError`] on directories failing to create.
///
/// # Side Effects
/// [Storage: Filesystem]
pub async fn object_update_cached(
    objects: Vec<CachedObjectData>,
    cache: Cache,
    region_id: String,
    out_dir: PathBuf,
) -> Result<Vec<ObjectUpdateAction>, ObjectUpdateError> {
    let mut cache_results = Vec::new();
    for object in objects {
        match cache
            .object
            .check_cache(object.id, object.crc, region_id.clone())
            .await
        {
            Ok((asset_id, json_path, glb, generator_object)) => {
                let base_dir = create_sub_object_dir(&out_dir, &asset_id.to_string())?;
                if let Some(mesh_path) = glb {
                    cache_results.push(ObjectUpdateAction::Render(RenderMeshData {
                        mesh_path: Some(mesh_path),
                        base_dir,
                        asset_id,
                        object: generator_object,
                        retry_count: 0,
                        download: None,
                    }));
                } else {
                    warn!(
                        "Generated mesh file {:?} not found. Generating now.",
                        asset_id
                    );
                    cache_results.push(ObjectUpdateAction::GenerateFromRenderObject(
                        RenderObjectJSONData {
                            json_path,
                            base_dir,
                            asset_id,
                            object: generator_object,
                        },
                    ));
                }
            }
            Err(_) => {
                cache_results.push(ObjectUpdateAction::HandleCacheMiss((
                    CacheMissType::Normal,
                    object.id,
                )));
            }
        };
    }
    Ok(cache_results)
}

/// TODO: currently a stub
pub async fn handle_attachment(
    _attachment: AttachItem,
    object: AttachmentObjectData,
    cache: Cache,
) -> Result<Vec<ObjectUpdateAction>, ObjectUpdateError> {
    let actions: Vec<ObjectUpdateAction> = Vec::new();
    let mut current_id = match object.parent_id {
        Some(id) => id,
        None => return Ok(actions),
    };

    let mut visited = std::collections::HashSet::new();

    loop {
        if !visited.insert(current_id) {
            break;
        }

        let parent = match cache.object.get_parent(current_id).await {
            Ok(p) => p,
            Err(_) => return Ok(actions),
        };

        match parent {
            Some(p) => {
                current_id = p;
            }
            None => break,
        }
    }

    Err(ObjectUpdateError::Unimplemented {
        feature: "Attach Items".to_string(),
    })
}

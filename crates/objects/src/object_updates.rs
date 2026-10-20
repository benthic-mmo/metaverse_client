use crate::errors::ObjectUpdateError;
use benthic_protocol::{
    objects::{
        objects::{AttachmentObjectData, ObjectInstanceInfo, RetriableData},
        types::TextureAnimation,
    },
    session::{session::create_sub_object_dir, types::DownloadState},
};
use log::warn;
use metaverse_messages::{
    udp::object::{
        object_update::AttachItem, object_update_cached::CachedObjectData,
        request_multiple_objects::CacheMissType,
    },
    utils::{object_types::ObjectType, texture_entry::TextureEntries},
};
use metaverse_store::initialize_sqlite::Cache;
use std::{collections::HashMap, path::PathBuf, sync::Arc};
use tokio::sync::{
    Mutex,
    watch::{self, Sender},
};
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
    pub download: Option<watch::Receiver<DownloadState>>,
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
}
impl RetriableData for MeshObjectData {
    fn retry_count(&self) -> u32 {
        self.retry_count
    }
    fn inc_retry_count(&mut self) {
        self.retry_count += 1
    }
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

/// Determines which actions to take when receiving a mesh update
///
/// If the mesh's data is already cached, render directly.
/// Check if the download is in progress, and queue the render mesh data objects behind it.
/// If no download is in progress, create a new download to run in the background
///
/// # Parameters
/// - `cache`: The session's db cache
/// - `object`: data required for determining the state of the mesh object's download
/// - `out_dir`: Root dir for the mesh's gltf and JSON data
///
/// # Errors
/// Returns [`ObjectUpdateError`] when the base dir fails to create, or the cache fails to update
///
/// # Side Effects
/// [Storage: Filesystem + SQLite]
pub async fn mesh_update(
    cache: Cache,
    object: MeshObjectData,
    out_dir: PathBuf,
    downloads: Arc<Mutex<HashMap<Uuid, Sender<DownloadState>>>>,
) -> Result<Vec<ObjectUpdateAction>, ObjectUpdateError> {
    cache
        .object
        .update(
            object.object_info.clone(),
            object.crc,
            object.region_id.clone(),
            object.object_type,
        )
        .await?;
    let mut actions = Vec::new();
    match cache
        .object
        .check_cache_by_asset_id(object.sculpt_id, object.object_info.full_id)
        .await
    {
        // If we have the glb path already, render it right away.
        Ok((glb_path, generator)) => {
            let base_dir =
                create_sub_object_dir(&out_dir, &object.object_info.full_id.to_string())?;

            actions.push(ObjectUpdateAction::Render(RenderMeshData {
                mesh_path: Some(glb_path),
                base_dir,
                asset_id: object.object_info.full_id,
                object: generator,
                retry_count: 0,
                download: None,
            }));
        }

        Err(_) => {
            // check to see if the download is in progress
            let mut downloads = downloads.lock().await;
            if let Some(tx) = downloads.get(&object.sculpt_id) {
                // there is a download in progress.
                // Create a RenderObject with a subscriber.
                actions.push(ObjectUpdateAction::Render(RenderMeshData {
                    mesh_path: None,
                    base_dir: out_dir.clone(),
                    asset_id: object.object_info.full_id,
                    object: object.object_info,
                    retry_count: 0,
                    download: Some(tx.subscribe()),
                }));
            } else {
                // This is the first request for the asset.
                // create a DownloadObject with a sender.
                let (tx, _rx) = tokio::sync::watch::channel(DownloadState::Downloading);
                downloads.insert(object.sculpt_id, tx);
                drop(downloads);

                actions.push(ObjectUpdateAction::DownloadMesh(object));
            }
        }
    };
    Ok(actions)
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

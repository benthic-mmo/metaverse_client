use std::{collections::HashMap, path::PathBuf, sync::Arc};

use benthic_protocol::{
    objects::{AttachmentObjectData, GeneratorObject, MeshObjectData},
    session::{DownloadState, create_sub_object_dir},
};
use glam::Vec3;
use log::warn;
use metaverse_avatar::avatar::Avatar;
use metaverse_messages::{
    udp::object::{
        object_update::AttachItem, object_update_cached::CachedObjectData,
        request_multiple_objects::CacheMissType,
    },
    utils::{object_types::ObjectType, texture_entry::TextureEntry},
};
use metaverse_store::initialize_sqlite::Cache;
use tokio::sync::{
    Mutex,
    watch::{self, Sender},
};
use uuid::Uuid;

use crate::errors::ObjectUpdateError;

#[derive(Debug)]
pub struct RenderObjectData {
    pub mesh_path: Option<PathBuf>,
    pub base_dir: PathBuf,
    pub asset_id: Uuid,
    pub object: GeneratorObject,
    pub retry_count: u32,
    pub download: Option<watch::Receiver<DownloadState>>,
}

#[derive(Debug)]
pub struct GenerateMeshData {
    pub json_path: PathBuf,
    pub base_dir: PathBuf,
    pub asset_id: Uuid,
    pub object: GeneratorObject,
}

#[derive(Debug)]
pub struct NewAvatarData {
    pub avatar: Avatar,
    pub retry_count: u32,
}

pub enum CacheResult {
    RenderObject(RenderObjectData),
    GenerateObect(GenerateMeshData),
    Miss((CacheMissType, u32)),
}

#[derive(Debug)]
pub struct DownloadMeshObjectData {
    pub object: MeshObjectData<TextureEntry, ObjectType>,
    pub retry_count: u32,
}

pub struct ParametricPrimRenderData {
    pub vertices: Vec<Vec3>,
    pub indices: Vec<u32>,
}

pub enum ObjectUpdateAction {
    DownloadMesh(DownloadMeshObjectData),
    Render(RenderObjectData),
    NewAvatar(NewAvatarData),
    GenerateFromJSON(GenerateMeshData),
    HandleCacheMiss((CacheMissType, u32)),
}

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
                    cache_results.push(ObjectUpdateAction::Render(RenderObjectData {
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
                    cache_results.push(ObjectUpdateAction::GenerateFromJSON(GenerateMeshData {
                        json_path,
                        base_dir,
                        asset_id,
                        object: generator_object,
                    }));
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

pub async fn mesh_update(
    cache: Cache,
    object: MeshObjectData<TextureEntry, ObjectType>,
    out_dir: PathBuf,
    downloads: Arc<Mutex<HashMap<Uuid, Sender<DownloadState>>>>,
) -> Result<Vec<ObjectUpdateAction>, ObjectUpdateError> {
    cache.object.update(object.clone()).await?;
    let mut actions = Vec::new();

    match cache
        .object
        .check_cache_by_asset_id(object.sculpt_id, object.full_id)
        .await
    {
        // If we have the glb path already, render it right away.
        Ok((glb_path, generator)) => {
            let base_dir = create_sub_object_dir(&out_dir, &object.full_id.to_string())?;

            actions.push(ObjectUpdateAction::Render(RenderObjectData {
                mesh_path: Some(glb_path),
                base_dir,
                asset_id: object.full_id,
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
                actions.push(ObjectUpdateAction::Render(RenderObjectData {
                    mesh_path: None,
                    base_dir: out_dir.clone(),
                    asset_id: object.full_id,
                    object: GeneratorObject {
                        full_id: object.full_id,
                        local_id: object.local_id,
                        parent_id: object.parent,
                        position: object.position,
                        scale: object.scale,
                        rotation: object.rotation,
                    },
                    retry_count: 0,
                    download: Some(tx.subscribe()),
                }));
            } else {
                // This is the first request for the asset.
                // create a DownloadObject with a sender.
                let (tx, _rx) = tokio::sync::watch::channel(DownloadState::Downloading);
                downloads.insert(object.sculpt_id, tx);
                drop(downloads);

                actions.push(ObjectUpdateAction::DownloadMesh(DownloadMeshObjectData {
                    object: object.clone(),
                    retry_count: 0,
                }));
            }
        }
    };
    Ok(actions)
}

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

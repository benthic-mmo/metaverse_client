use std::{collections::HashMap, path::PathBuf, sync::Arc};

use benthic_protocol::{
    objects::{GeneratorObject, MinimalObjectUpdate},
    session::{DownloadState, create_sub_agent_dir, create_sub_object_dir},
};
use log::warn;
use metaverse_avatar::avatar::Avatar;
use metaverse_messages::{
    http::scene::SculptType,
    udp::object::{
        object_update::{AttachItem, ExtraParams},
        object_update_cached::CachedObjectData,
        request_multiple_objects::CacheMissType,
    },
    utils::{object_types::ObjectType, texture_entry::TextureEntry},
};
use metaverse_store::initialize_sqlite::Cache;
use tokio::sync::{
    Mutex,
    watch::{self, Receiver, Sender},
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
pub struct DownloadObjectData {
    pub object: MinimalObjectUpdate<ExtraParams, TextureEntry, ObjectType>,
    pub asset_id: Uuid,
    pub texture_id: Uuid,
    pub retry_count: u32,
}

pub enum ObjectUpdateAction {
    Download(DownloadObjectData),
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
            Err(e) => {
                cache_results.push(ObjectUpdateAction::HandleCacheMiss((
                    CacheMissType::Normal,
                    object.id,
                )));
            }
        };
    }
    Ok(cache_results)
}

pub async fn object_update(
    cache: Cache,
    object: MinimalObjectUpdate<ExtraParams, TextureEntry, ObjectType>,
    out_dir: PathBuf,
    downloads: Arc<Mutex<HashMap<Uuid, Sender<DownloadState>>>>,
) -> Result<Vec<ObjectUpdateAction>, ObjectUpdateError> {
    cache.object.update(object.clone()).await?;
    let actions = match object.object_type {
        ObjectType::Prim => handle_prim(object, cache, out_dir, downloads).await?,
        ObjectType::Tree => handle_tree(object)?,
        ObjectType::Grass => handle_grass(object)?,
        ObjectType::Unknown => handle_unknown(object)?,
        ObjectType::ParticleSystem => handle_particle_system(object)?,
        ObjectType::NewTree => handle_new_tree(object)?,
        ObjectType::Avatar => handle_avatar(object, out_dir)?,
        object_type => Err(ObjectUpdateError::UnknownObjectError {
            object_type: object_type.to_string(),
        })?,
    };
    Ok(actions)
}

async fn handle_prim(
    object: MinimalObjectUpdate<ExtraParams, TextureEntry, ObjectType>,
    cache: Cache,
    out_dir: PathBuf,
    downloads: Arc<Mutex<HashMap<Uuid, Sender<DownloadState>>>>,
) -> Result<Vec<ObjectUpdateAction>, ObjectUpdateError> {
    if let Some(name) = object.name_value.as_ref()
        && let Ok(attachment) = AttachItem::parse_attach_item(name)
    {
        return handle_attachment(attachment, object, cache).await;
    }
    let mut actions: Vec<ObjectUpdateAction> = Vec::new();
    if let Some(extra_params) = &object.extra_params {
        for param in extra_params {
            match param {
                ExtraParams::Sculpt(sculpt) => match sculpt.sculpt_type {
                    SculptType::Mesh => {
                        // check the cache. If it has a glb path, senda renderobjectdata message instead.
                        match cache
                            .object
                            .check_cache_by_asset_id(sculpt.texture_id, object.full_id)
                            .await
                        {
                            // If we have the glb path already, render it right away.
                            Ok((glb_path, generator)) => {
                                let base_dir =
                                    create_sub_object_dir(&out_dir, &object.full_id.to_string())?;

    println!(
        "CACHE HIT!!!!!!!!!!!!!!!: pushing RenderObject for object {} asset {} path {:?}",
        object.full_id,
        sculpt.texture_id,
        glb_path
    );
                                actions.push(ObjectUpdateAction::Render(RenderObjectData {
                                    mesh_path: Some(glb_path),
                                    base_dir,
                                    asset_id: object.full_id,
                                    object: generator,
                                    retry_count: 0,
                                    download: None,
                                }));
                            }

                            Err(e) => {
                                // check to see if the download is in progress
                                let mut downloads = downloads.lock().await;
                                if let Some(tx) = downloads.get(&sculpt.texture_id) {
                                    // there is a download in progress.
                                    // Create a RenderObject with a subscriber.
                                    actions.push(ObjectUpdateAction::Render(RenderObjectData {
                                        mesh_path: None,
                                        base_dir: out_dir.clone(),
                                        asset_id: object.full_id,
                                        object: GeneratorObject {
                                            full_id: object.full_id,
                                            local_id: object.local_id,
                                            parent_id: object.parent_id,
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
                                    let (tx, rx) =
                                        tokio::sync::watch::channel(DownloadState::Downloading);
                                    downloads.insert(sculpt.texture_id, tx);
                                    drop(downloads);

                                    actions.push(ObjectUpdateAction::Download(
                                        DownloadObjectData {
                                            asset_id: sculpt.texture_id,
                                            texture_id: object.texture.texture_id,
                                            object: object.clone(),
                                            retry_count: 0,
                                        },
                                    ));
                                }
                            }
                        };
                    }
                    _ => {
                        Err(ObjectUpdateError::Unimplemented {
                            feature: "Non-Mesh sculpt type".to_string(),
                        })?;
                    }
                },
                _ => Err(ObjectUpdateError::Unimplemented {
                    feature: "Non-Sculpt object update".to_string(),
                })?,
            };
        }
    }

    Ok(actions)
}

pub async fn handle_attachment(
    _attachment: AttachItem,
    object: MinimalObjectUpdate<ExtraParams, TextureEntry, ObjectType>,
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

fn handle_tree(
    _object: MinimalObjectUpdate<ExtraParams, TextureEntry, ObjectType>,
) -> Result<Vec<ObjectUpdateAction>, ObjectUpdateError> {
    Err(ObjectUpdateError::Unimplemented {
        feature: "Trees".to_string(),
    })
}
fn handle_grass(
    _object: MinimalObjectUpdate<ExtraParams, TextureEntry, ObjectType>,
) -> Result<Vec<ObjectUpdateAction>, ObjectUpdateError> {
    Err(ObjectUpdateError::Unimplemented {
        feature: "Grass".to_string(),
    })
}
fn handle_unknown(
    _object: MinimalObjectUpdate<ExtraParams, TextureEntry, ObjectType>,
) -> Result<Vec<ObjectUpdateAction>, ObjectUpdateError> {
    Err(ObjectUpdateError::Unimplemented {
        feature: "Unknown".to_string(),
    })
}
fn handle_particle_system(
    _object: MinimalObjectUpdate<ExtraParams, TextureEntry, ObjectType>,
) -> Result<Vec<ObjectUpdateAction>, ObjectUpdateError> {
    Err(ObjectUpdateError::Unimplemented {
        feature: "Partilce System".to_string(),
    })
}
fn handle_new_tree(
    _object: MinimalObjectUpdate<ExtraParams, TextureEntry, ObjectType>,
) -> Result<Vec<ObjectUpdateAction>, ObjectUpdateError> {
    Err(ObjectUpdateError::Unimplemented {
        feature: "New Tree".to_string(),
    })
}

fn handle_avatar(
    object: MinimalObjectUpdate<ExtraParams, TextureEntry, ObjectType>,
    out_dir: PathBuf,
) -> Result<Vec<ObjectUpdateAction>, ObjectUpdateError> {
    create_sub_agent_dir(&out_dir, &object.full_id.to_string())?;
    Ok(vec![ObjectUpdateAction::NewAvatar(NewAvatarData {
        avatar: Avatar::new(object.full_id, object.position),
        retry_count: 0,
    })])
}

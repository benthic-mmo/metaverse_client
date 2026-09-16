use std::path::PathBuf;

use benthic_protocol::{
    objects::{GeneratorObject, MinimalObjectUpdate},
    session::{create_sub_agent_dir, create_sub_object_dir},
};
use log::{info, warn};
use metaverse_avatar::avatar::Avatar;
use metaverse_cache::object_update::{
    sqlite_check_cache, sqlite_get_parent, sqlite_insert_object_update,
};
use metaverse_messages::{
    udp::object::{
        object_update::{AttachItem, ExtraParams},
        object_update_cached::CachedObjectData,
        request_multiple_objects::CacheMissType,
    },
    utils::{object_types::ObjectType, texture_entry::TextureEntry},
};
use sqlx::{Pool, Sqlite};
use uuid::Uuid;

use crate::{errors::ObjectUpdateError, object_updates::ObjectUpdateAction::NewAvatar};

#[derive(Debug)]
pub struct RenderObjectData {
    pub mesh_path: PathBuf,
    pub base_dir: PathBuf,
    pub asset_id: Uuid,
    pub object: GeneratorObject,
    pub retry_count: u32,
}

#[derive(Debug)]
pub struct GenerateMeshData {
    pub json_path: PathBuf,
    pub base_dir: PathBuf,
    pub asset_id: Uuid,
    pub object: GeneratorObject,
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
}

pub enum ObjectUpdateAction {
    Download(DownloadObjectData),
    Render(RenderObjectData),
    NewAvatar(Avatar),
    GenerateFromJSON(GenerateMeshData),
    HandleCacheMiss((CacheMissType, u32)),
}

pub async fn object_update_cached(
    objects: Vec<CachedObjectData>,
    db_pool: &Pool<Sqlite>,
    region_id: String,
) -> Result<Vec<ObjectUpdateAction>, ObjectUpdateError> {
    let mut cache_results = Vec::new();
    for object in objects {
        match sqlite_check_cache(db_pool, object.id, object.crc, region_id.clone()).await {
            Ok((asset_id, json_path, glb, generator_object)) => {
                let base_dir = create_sub_object_dir(&asset_id.to_string())?;
                if let Some(mesh_path) = glb {
                    cache_results.push(ObjectUpdateAction::Render(RenderObjectData {
                        mesh_path,
                        base_dir,
                        asset_id,
                        object: generator_object,
                        retry_count: 0,
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
                info!(
                    "Cache did not contain {}, {} from {}, {:?}",
                    object.id, object.crc, region_id, e
                );
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
    db_conn: &Pool<Sqlite>,
    object: MinimalObjectUpdate<ExtraParams, TextureEntry, ObjectType>,
) -> Result<Vec<ObjectUpdateAction>, ObjectUpdateError> {
    sqlite_insert_object_update(db_conn, object.clone()).await?;
    let actions = match object.object_type {
        ObjectType::Prim => handle_prim(object, db_conn).await?,
        ObjectType::Tree => handle_tree(object)?,
        ObjectType::Grass => handle_grass(object)?,
        ObjectType::Unknown => handle_unknown(object)?,
        ObjectType::ParticleSystem => handle_particle_system(object)?,
        ObjectType::NewTree => handle_new_tree(object)?,
        ObjectType::Avatar => handle_avatar(object)?,
        object_type => Err(ObjectUpdateError::UnknownObjectError {
            object_type: object_type.to_string(),
        })?,
    };
    Ok(actions)
}

async fn handle_prim(
    object: MinimalObjectUpdate<ExtraParams, TextureEntry, ObjectType>,
    db_conn: &Pool<Sqlite>,
) -> Result<Vec<ObjectUpdateAction>, ObjectUpdateError> {
    if let Some(name) = object.name_value.as_ref()
        && let Ok(attachment) = AttachItem::parse_attach_item(name)
    {
        return handle_attachment(attachment, object, db_conn).await;
    }
    let mut actions: Vec<ObjectUpdateAction> = Vec::new();
    if let Some(extra_params) = &object.extra_params {
        for param in extra_params {
            match param {
                ExtraParams::Sculpt(sculpt) => {
                    actions.push(ObjectUpdateAction::Download(DownloadObjectData {
                        asset_id: sculpt.texture_id,
                        texture_id: object.texture.texture_id,
                        object: object.clone(),
                    }))
                }
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
    db_conn: &Pool<Sqlite>,
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

        let parent = match sqlite_get_parent(db_conn, current_id).await {
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
    object: MinimalObjectUpdate<ExtraParams, TextureEntry, ObjectType>,
) -> Result<Vec<ObjectUpdateAction>, ObjectUpdateError> {
    Err(ObjectUpdateError::Unimplemented {
        feature: "Trees".to_string(),
    })
}
fn handle_grass(
    object: MinimalObjectUpdate<ExtraParams, TextureEntry, ObjectType>,
) -> Result<Vec<ObjectUpdateAction>, ObjectUpdateError> {
    Err(ObjectUpdateError::Unimplemented {
        feature: "Grass".to_string(),
    })
}
fn handle_unknown(
    object: MinimalObjectUpdate<ExtraParams, TextureEntry, ObjectType>,
) -> Result<Vec<ObjectUpdateAction>, ObjectUpdateError> {
    Err(ObjectUpdateError::Unimplemented {
        feature: "Unknown".to_string(),
    })
}
fn handle_particle_system(
    object: MinimalObjectUpdate<ExtraParams, TextureEntry, ObjectType>,
) -> Result<Vec<ObjectUpdateAction>, ObjectUpdateError> {
    Err(ObjectUpdateError::Unimplemented {
        feature: "Partilce System".to_string(),
    })
}
fn handle_new_tree(
    object: MinimalObjectUpdate<ExtraParams, TextureEntry, ObjectType>,
) -> Result<Vec<ObjectUpdateAction>, ObjectUpdateError> {
    Err(ObjectUpdateError::Unimplemented {
        feature: "New Tree".to_string(),
    })
}

fn handle_avatar(
    object: MinimalObjectUpdate<ExtraParams, TextureEntry, ObjectType>,
) -> Result<Vec<ObjectUpdateAction>, ObjectUpdateError> {
    create_sub_agent_dir(&object.full_id.to_string())?;
    Ok(vec![NewAvatar(Avatar::new(
        object.full_id,
        object.position,
    ))])
}

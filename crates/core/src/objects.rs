use crate::COORDINATE_CONVERSION;
use crate::actors::SessionActor;
use crate::avatar::AddObjectToAvatar;
use crate::session::OutgoingPacket;
use crate::session::RetryMessage;
use crate::ui_interactions::SendUIMessage;
use actix::Addr;
use actix::AsyncContext;
use actix::ResponseFuture;
use actix::WrapFuture;
use actix::{Handler, Message};
use benthic_default_asset_converter::default_texture_path;
use benthic_default_asset_converter::no_geometry_mesh_path;
use benthic_protocol::avatar::types::AttachmentData;
use benthic_protocol::messages::simulator::orphan_objects::OrphanedObject;
use benthic_protocol::messages::ui_protocol::UIMessage;
use benthic_protocol::messages::world::mesh_update::MeshType;
use benthic_protocol::messages::world::mesh_update::MeshUpdate;
use benthic_protocol::objects::types::ObjectInstanceInfo;
use benthic_protocol::objects::types::RetriableData;
use benthic_protocol::session::storage::AgentId;
use benthic_protocol::session::storage::BenthicDirs;
use benthic_protocol::session::storage::ObjectId;
use benthic_protocol::session::types::AssetKey;
use benthic_protocol::session::types::DownloadState;
use futures::future::join_all;
use log::warn;
use log::{error, info};
use metaverse_avatar::avatar::Avatar;
use metaverse_messages::http::capabilities::Capability;
use metaverse_messages::packet::packet_protocol::Packet;
use metaverse_messages::udp::object::improved_terse_object_update::ImprovedTerseObjectUpdate;
use metaverse_messages::udp::object::object_update::MaterialsData;
use metaverse_messages::udp::object::object_update_cached::ObjectUpdateCached;
use metaverse_messages::udp::object::request_multiple_objects::RequestMultipleObjects;
use metaverse_messages::udp::object::util::PBRMaterial;
use metaverse_messages::utils::texture_entry::TextureEntries;
use metaverse_objects::download_asset::download_material;
use metaverse_objects::download_asset::download_mesh;
use metaverse_objects::download_asset::download_texture;
use metaverse_objects::errors::DownloadError;
use metaverse_objects::mesh::handle_mesh_avatar_object;
use metaverse_objects::mesh::handle_mesh_object;
use metaverse_objects::mesh::mesh_from_json;
use metaverse_objects::object_updates::AvatarAttachment;
use metaverse_objects::object_updates::CachedObjectUpdateType;
use metaverse_objects::object_updates::MeshObjectData;
use metaverse_objects::object_updates::ObjectUpdateAction;
use metaverse_objects::object_updates::ParametricPrimData;
use metaverse_objects::object_updates::RenderMeshData;
use metaverse_objects::object_updates::RenderObjectJSONData;
use metaverse_objects::object_updates::SculptObjectData;
use metaverse_objects::object_updates::object_update_cached;
use metaverse_objects::parametric_prim::handler::handle_parametric_prim_avatar_object;
use metaverse_objects::parametric_prim::handler::handle_parametric_prim_object;
use metaverse_objects::sculpt_object::handle_sculpt_object;
use metaverse_store::cache::object::ObjectCache;
use metaverse_store::errors::InventoryError;
use std::collections::HashMap;
use std::collections::HashSet;
use std::path::PathBuf;
use tokio::sync::watch;
use uuid::Uuid;

/// Message for handling object updates of type mesh
///
/// This message handles the whole process of resolving the assets required to build a mesh.
/// This message waits on each download using the async await, only re-awakening after all of the
/// required objects have fully downloaded.
///
/// # Cause
/// - Object update with type Mesh comes in from the udp packet reader
///
/// # Effect
/// [`GetAsset`] via [`resolve_asset`]
/// [`RenderBlankMeshMessage`] if the mesh download fails
/// [`RenderObjectMessage`]

#[derive(Message, Clone)]
#[rtype(result = "()")]
pub struct MeshObjectUpdate(pub MeshObjectData);

impl Handler<MeshObjectUpdate> for SessionActor {
    type Result = ();
    fn handle(&mut self, mut msg: MeshObjectUpdate, ctx: &mut Self::Context) {
        let Some(session) = self.session.as_mut() else {
            return;
        };

        let cache = self.cache.clone();
        let dirs = self.session_context.dirs.clone();
        let addr = ctx.address();
        let avatars = session.avatars.clone();

        ctx.spawn(
            async move {
                if let Some(parent_id) = msg.0.object_info.parent_id
                    && let Some(attachment) = try_attach_to_avatar(
                        &cache.object,
                        parent_id,
                        &CachedObjectUpdateType::Mesh(msg.0.clone()),
                        &avatars,
                    )
                    .await
                {
                    msg.0.avatar_attachment = Some(attachment);
                }

                let (is_avatar, base_dir, gltf_path) =
                    if let Some(av) = msg.0.avatar_attachment.as_ref() {
                        let agent = AgentId(av.agent_id);
                        let dir = match dirs.ensure_agent_dir(agent) {
                            Ok(d) => d,
                            Err(err) => {
                                error!("MeshObjectUpdate failed: {:?}", err);
                                return;
                            }
                        };
                        (true, dir, dirs.agent_high_gltf(agent))
                    } else {
                        let sculpt = ObjectId(msg.0.sculpt_id);
                        let dir = match dirs.ensure_object_dir(sculpt) {
                            Ok(d) => d,
                            Err(err) => {
                                error!("MeshObjectUpdate failed: {:?}", err);
                                return;
                            }
                        };
                        (false, dir, dirs.object_high_gltf(sculpt))
                    };

                if !is_avatar
                    && tokio::fs::try_exists(&gltf_path)
                        .await
                        .unwrap_or_else(|err| {
                            warn!("Failed to check glTF existence: {:?}", err);
                            false
                        })
                {
                    addr.do_send(RenderObjectMessage(RenderMeshData {
                        asset_id: msg.0.sculpt_id,
                        object: msg.0.object_info,
                        mesh_path: Some(gltf_path),
                        base_dir,
                        retry_count: 0,
                        download: None,
                    }));
                    return;
                }

                if let Err(err) = cache_object_update(
                    &cache.object,
                    CachedObjectUpdateType::Mesh(msg.0.clone()),
                    &dirs,
                )
                .await
                {
                    error!("Failed to cache object update: {err}");
                }

                let mesh_future = resolve_asset(addr.clone(), AssetKey::Mesh(msg.0.sculpt_id));
                let texture_future = face_textures_or_fallback(&addr, &msg.0.texture);
                let materials_future = face_pbr_materials(&addr, &msg.0.pbr_materials);

                let (mesh_result, texture_paths, material_paths) =
                    tokio::join!(mesh_future, texture_future, materials_future);

                let mesh_path = match mesh_result {
                    Ok(path) => path,
                    Err(e) => {
                        error!("Mesh download failed: {e}");
                        addr.do_send(RenderBlankMeshMessage {
                            dirs: dirs.clone(),
                            asset_id: msg.0.sculpt_id,
                            object: msg.0.object_info,
                            avatar_attachment: msg.0.avatar_attachment,
                        });
                        return;
                    }
                };

                let mesh_bytes = match tokio::fs::read(&mesh_path).await {
                    Ok(bytes) => bytes.into(),
                    Err(e) => {
                        error!("Failed to read mesh {}: {e}", msg.0.sculpt_id);
                        return;
                    }
                };

                if is_avatar {
                    let action = match handle_mesh_avatar_object(
                        cache,
                        &msg.0,
                        &dirs,
                        mesh_bytes,
                        texture_paths,
                        material_paths,
                    )
                    .await
                    {
                        Ok(a) => a,
                        Err(e) => {
                            error!("Mesh object processing failed: {e:?}");
                            return;
                        }
                    };
                    if let ObjectUpdateAction::AvatarObject(action) = action {
                        addr.do_send(AddObjectToAvatar(action));
                    }
                } else {
                    let action = match handle_mesh_object(
                        cache,
                        &msg.0,
                        &dirs,
                        mesh_bytes,
                        texture_paths,
                        material_paths,
                    )
                    .await
                    {
                        Ok(a) => a,
                        Err(e) => {
                            error!("Mesh object processing failed: {e:?}");
                            return;
                        }
                    };
                    if let ObjectUpdateAction::Render(mut action) = action {
                        action.download = None;
                        addr.do_send(RenderObjectMessage(action));
                    }
                }
            }
            .into_actor(self),
        );
    }
}

/// Message for handling object updates of type parametric prim
///
/// This message handles the whole process of resolving the assets required to build a parametric prim.
/// This message waits on each download using the async await, only re-awakening after all of the
/// required objects have fully downloaded.
///
/// # Cause
/// - Object update with type Parametric Prim comes in from the udp packet reader
///
/// # Effect
/// [`GetAsset`] via [`resolve_asset`]
/// [`RenderBlankMeshMessage`] if the mesh download fails
/// [`RenderObjectMessage`]
#[derive(Message, Clone)]
#[rtype(result = "()")]
pub struct ParametricPrimObjectUpdate(pub ParametricPrimData);
impl Handler<ParametricPrimObjectUpdate> for SessionActor {
    type Result = ();

    fn handle(&mut self, mut msg: ParametricPrimObjectUpdate, ctx: &mut Self::Context) {
        let Some(session) = self.session.as_ref() else {
            return;
        };

        let dirs = self.session_context.dirs.clone();
        let addr = ctx.address();
        let avatars = session.avatars.clone();
        let cache = self.cache.clone();

        ctx.spawn(
            async move {
                if let Some(parent_id) = msg.0.object_info.parent_id
                    && let Some(attachment) = try_attach_to_avatar(
                        &cache.object,
                        parent_id,
                        &CachedObjectUpdateType::Prim(msg.0.clone()),
                        &avatars,
                    )
                    .await
                {
                    msg.0.avatar_attachment = Some(attachment);
                }

                if let Err(err) = cache_object_update(
                    &cache.object,
                    CachedObjectUpdateType::Prim(msg.0.clone()),
                    &dirs,
                )
                .await
                {
                    error!("Failed to cache object update: {err}");
                }

                let texture_future = face_textures_or_fallback(&addr, &msg.0.texture);
                let materials_future = face_pbr_materials(&addr, &msg.0.pbr_materials);
                let (texture_paths, material_paths) =
                    tokio::join!(texture_future, materials_future);

                if let Some(_avatar_data) = msg.0.avatar_attachment.as_ref() {
                    handle_parametric_prim_avatar_object().await;
                } else {
                    let actions = match handle_parametric_prim_object(
                        &dirs,
                        &msg.0,
                        texture_paths,
                        material_paths,
                        msg.0.object_info.full_id,
                    )
                    .await
                    {
                        Ok(a) => a,
                        Err(e) => {
                            error!("Parametric prim processing failed: {e:?}");
                            addr.do_send(RenderBlankMeshMessage {
                                dirs: dirs.clone(),
                                asset_id: msg.0.object_info.full_id,
                                object: msg.0.object_info,
                                avatar_attachment: msg.0.avatar_attachment,
                            });
                            return;
                        }
                    };
                    for action in actions {
                        if let ObjectUpdateAction::Render(data) = action {
                            addr.do_send(RenderObjectMessage(data));
                        }
                    }
                }
            }
            .into_actor(self),
        );
    }
}

/// Message for handling object updates of type sculpt object
///
/// This message handles the whole process of resolving the assets required to build a sculpt object.
/// This message waits on each download using the async await, only re-awakening after all of the
/// required objects have fully downloaded.
///
/// # Cause
/// - Object update with type Sculpt Object comes in from the udp packet reader
///
/// # Effect
/// [`GetAsset`] via [`resolve_asset`]
/// [`RenderBlankMeshMessage`] if the mesh download fails
/// [`RenderObjectMessage`]
#[derive(Debug, Message, Clone)]
#[rtype(result = "()")]
pub struct SculptObjectUpdate(pub SculptObjectData);
impl Handler<SculptObjectUpdate> for SessionActor {
    type Result = ();

    fn handle(&mut self, mut msg: SculptObjectUpdate, ctx: &mut Self::Context) {
        let Some(session) = self.session.as_ref() else {
            return;
        };

        let dirs = self.session_context.dirs.clone();
        let addr = ctx.address();
        let avatars = session.avatars.clone();
        let cache = self.cache.clone();

        ctx.spawn(
            async move {
                if let Some(parent_id) = msg.0.object_info.parent_id
                    && let Some(attachment) = try_attach_to_avatar(
                        &cache.object,
                        parent_id,
                        &CachedObjectUpdateType::Sculpt(msg.0.clone()),
                        &avatars,
                    )
                    .await
                {
                    msg.0.avatar_attachment = Some(attachment);
                }

                if let Err(err) = cache_object_update(
                    &cache.object,
                    CachedObjectUpdateType::Sculpt(msg.0.clone()),
                    &dirs,
                )
                .await
                {
                    error!("Failed to cache object update: {err}");
                }

                let sculpt_id = msg.0.data.texture_id;
                let sculpt_future = resolve_asset(addr.clone(), AssetKey::Texture(sculpt_id));
                let materials_future = face_pbr_materials(&addr, &msg.0.pbr_materials);
                let face_textures_future = face_textures_or_fallback(&addr, &msg.0.texture);

                let (sculpt_result, texture_paths, material_paths) =
                    tokio::join!(sculpt_future, face_textures_future, materials_future);

                let sculpt_path = match sculpt_result {
                    Ok(path) => path,
                    Err(e) => {
                        error!("Sculpt texture download failed: {e}");
                        addr.do_send(RenderBlankMeshMessage {
                            dirs: dirs.clone(),
                            asset_id: sculpt_id,
                            object: msg.0.object_info,
                            avatar_attachment: msg.0.avatar_attachment,
                        });
                        return;
                    }
                };

                if let Some(_avatar_data) = msg.0.avatar_attachment.as_ref() {
                    // deferred until avatar finalize
                } else {
                    let actions = match handle_sculpt_object(
                        &dirs,
                        &sculpt_path,
                        &msg.0,
                        texture_paths,
                        material_paths,
                        sculpt_id,
                    )
                    .await
                    {
                        Ok(a) => a,
                        Err(e) => {
                            error!("Sculpt object processing failed: {e:?}");
                            addr.do_send(RenderBlankMeshMessage {
                                dirs: dirs.clone(),
                                asset_id: sculpt_id,
                                object: msg.0.object_info,
                                avatar_attachment: msg.0.avatar_attachment,
                            });
                            return;
                        }
                    };
                    for action in actions {
                        if let ObjectUpdateAction::Render(data) = action {
                            addr.do_send(RenderObjectMessage(data));
                        }
                    }
                }
            }
            .into_actor(self),
        );
    }
}

/// Message for handling improved terse object update packets
///
/// TODO: currently unimplemented
///
/// # Cause
/// - ImprovedTerseObjectUpdate packet received from UDP socket
#[derive(Debug, Message)]
#[rtype(result = "()")]
pub struct HandleImprovedTerseObjectUpdate {
    /// The improved terse object update packet to handle
    pub improved_terse_object_update: ImprovedTerseObjectUpdate,
}
impl Handler<HandleImprovedTerseObjectUpdate> for SessionActor {
    type Result = ();
    fn handle(
        &mut self,
        _msg: HandleImprovedTerseObjectUpdate,
        _ctx: &mut Self::Context,
    ) -> Self::Result {
        // TODO: unimplemented
        //warn!("ImprovedTerseObjectUpdate packet received. Currently unimplemented.")
    }
}

/// Message for handling ObjectUpdateCached packets
///
/// Checks if the cache contains the objets in the objectupdatecached packet.
/// If they are already present, render them. If they aren't, request them from the server.
///
/// # Cause
/// - HandleObjectUpdateCached packet received from UDP socket
///
/// # Effect
/// If objects are not in cache
/// - [`RequestMultipleObjects`] sent to server
///
/// If objects are in cache
/// [`RenderObjectMessage`]
///
/// If only RenderObject JSON is in cache
/// [`GenerateMeshMessage`]
#[derive(Debug, Message)]
#[rtype(result = "()")]
pub struct HandleObjectUpdateCached {
    /// the object update cached packet to handle
    pub object_update_cached: ObjectUpdateCached,
}
impl Handler<HandleObjectUpdateCached> for SessionActor {
    type Result = ();
    fn handle(&mut self, msg: HandleObjectUpdateCached, ctx: &mut Self::Context) {
        let (Some(session), Some(&session_agent_id), Some(&session_id), Some(interaction_actor)) = (
            self.session.as_mut(),
            self.session_context.agent_id.get(),
            self.session_context.session_id.get(),
            self.interaction_actor.as_ref(),
        ) else {
            return;
        };

        let cache = self.cache.clone();
        let interaction_addr = interaction_actor.clone();
        let addr = ctx.address();
        let region_id = session.region_data.region_id.clone();
        let agent_id = session_agent_id;
        let dirs = self.session_context.dirs.clone();
        ctx.spawn(
            async move {
                match object_update_cached(
                    msg.object_update_cached.objects,
                    cache,
                    region_id,
                    &dirs,
                )
                .await
                {
                    Ok(cache_results) => {
                        let mut requests = Vec::new();
                        for cache_result in cache_results {
                            match cache_result {
                                ObjectUpdateAction::Render(data) => {
                                    addr.do_send(RenderObjectMessage(data));
                                }
                                ObjectUpdateAction::GenerateFromRenderObject(data) => {
                                    addr.do_send(GenerateMeshMessage(data));
                                }
                                ObjectUpdateAction::HandleCacheMiss(object) => {
                                    requests.push(object);
                                }
                                _ => {}
                            }
                        }
                        if !requests.is_empty() {
                            interaction_addr.do_send(OutgoingPacket {
                                packet: Packet::new_request_multiple_objects(
                                    RequestMultipleObjects {
                                        session_id,
                                        agent_id,
                                        requests,
                                    },
                                ),
                            });
                        }
                    }
                    Err(e) => {
                        error!("HandleObjectUpdateCached error: {:?}", e)
                    }
                };
            }
            .into_actor(self),
        );
    }
}

/// Helper message to generate mesh from stored RenderObject json
///
/// # Cause
/// [`HandleObjectUpdateCached`]
///
/// # Effect
/// [`RenderObjectMessage`]
#[derive(Debug, Message)]
#[rtype(result = "()")]
pub struct GenerateMeshMessage(pub RenderObjectJSONData);
impl Handler<GenerateMeshMessage> for SessionActor {
    type Result = ();
    fn handle(&mut self, msg: GenerateMeshMessage, ctx: &mut Self::Context) -> Self::Result {
        let cache = self.cache.clone();
        let addr = ctx.address();
        let downloads = self.downloads.clone();
        ctx.spawn(
            async move {
                match mesh_from_json(cache, msg.0).await {
                    Ok(action) => {
                        if let ObjectUpdateAction::Render(action) = action {
                            let Some(mesh_path) = action.mesh_path.clone() else {
                                error!("Generated mesh has no mesh path");
                                return;
                            };

                            let mut downloads = downloads.lock().await;

                            if let Some(tx) = downloads.remove(&AssetKey::Mesh(action.asset_id)) {
                                let _ = tx.send(DownloadState::Complete(mesh_path));
                            }

                            addr.do_send(RenderObjectMessage(action));
                        }
                    }
                    Err(e) => {
                        error!("GenerateMeshMessage error: {:?}", e)
                    }
                }
            }
            .into_actor(self),
        );
    }
}

/// Message for rendering objects to the UI
///
/// This contains specialized download wait logic, to sleep on downloading meshes until they are
/// ready to render.
///
/// # Cause
/// - [`GenerateMeshMessage`]
/// - [`ParametricPrimObjectUpdate`]
/// - [`SculptObjectUpdate`]
/// - [`HandleObjectUpdateCached`]
/// - [`GenerateMeshMessage`]
/// - [`RenderBlankMeshMessage`]
///
/// # Effect
/// [`MeshUpdate`]
#[derive(Debug, Message)]
#[rtype(result = "()")]
pub struct RenderObjectMessage(RenderMeshData);
impl Handler<RenderObjectMessage> for SessionActor {
    type Result = ();
    fn handle(&mut self, msg: RenderObjectMessage, _ctx: &mut Self::Context) -> Self::Result {
        let msg = msg.0;
        let Some(interaction_actor) = self.interaction_actor.as_ref() else {
            return;
        };

        let Some(mesh_path) = msg.mesh_path else {
            error!("No mesh path available for object {}", msg.object.full_id);
            return;
        };

        let (position, rotation) = if msg.object.parent_id.is_none_or(|id| id == 0) {
            (
                COORDINATE_CONVERSION.mul_vec3(msg.object.position),
                COORDINATE_CONVERSION * msg.object.rotation,
            )
        } else {
            (msg.object.position, msg.object.rotation)
        };

        info!("Rendering object {:?}", msg.asset_id);

        interaction_actor.do_send(SendUIMessage {
            ui_message: UIMessage::new_mesh_update(MeshUpdate {
                position,
                scale: msg.object.scale,
                rotation,
                parent: msg.object.parent_id,
                scene_id: Some(msg.object.local_id),
                path: mesh_path,
                mesh_type: MeshType::Object,
                full_id: Some(msg.object.full_id),
            }),
        });
    }
}

/// Message used for sending a blank GLTF to the UI to render.
///
/// this prevents parents from having orphaned children if their downloads fail.
/// Because of the parenting system in open metaverse applications, if the parent fails to download
/// in, the children still know where they are supposed to go. This tells the UI to create an object
/// with no visible mesh to ensure parenting stays consistent even with failed objects.
///
/// # Causes
/// [`DownloadMeshObject`]
/// [`ParametricPrimObjectUpdate`]
/// [`SculptObjectUpdate`]
#[derive(Debug, Message)]
#[rtype(result = "()")]
struct RenderBlankMeshMessage {
    /// base dir of the session. usually ~/.local/share/benthic
    dirs: BenthicDirs,
    /// ID of the object that failed
    asset_id: Uuid,
    /// common instance information such as location, scale, etc.
    object: ObjectInstanceInfo,

    avatar_attachment: Option<AvatarAttachment>,
}
impl Handler<RenderBlankMeshMessage> for SessionActor {
    type Result = ();
    fn handle(&mut self, msg: RenderBlankMeshMessage, ctx: &mut Self::Context) -> Self::Result {
        if let Some(_attachment_info) = msg.avatar_attachment {
            return;
        }

        let base_dir = match msg.dirs.ensure_empty_object_dir() {
            Ok(path) => path,
            Err(e) => {
                error!("Failed to create empty object dir: {e}");
                return;
            }
        };
        ctx.address().do_send(RenderObjectMessage(RenderMeshData {
            mesh_path: Some(no_geometry_mesh_path()),
            base_dir,
            asset_id: msg.asset_id,
            object: msg.object,
            retry_count: 0,
            download: None,
        }));
    }
}

async fn cache_object_update(
    cache: &ObjectCache,
    object: CachedObjectUpdateType,
    dirs: &BenthicDirs,
) -> Result<(), InventoryError> {
    let object_info = object.object_info().clone();
    let crc = object.crc();
    let region_id = object.region_id().to_owned();
    let object_type = object.object_type();

    // Use the asset ID as the directory key.
    let asset_id = match &object {
        CachedObjectUpdateType::Mesh(update) => update.sculpt_id,
        CachedObjectUpdateType::Sculpt(update) => update.data.texture_id,
        CachedObjectUpdateType::Prim(update) => update.object_info.full_id,
    };

    let asset_id = ObjectId(asset_id);

    dirs.ensure_object_updates_dir()?;

    let path = dirs.object_update_json(asset_id);

    let json = serde_json::to_string_pretty(&object)?;
    tokio::fs::write(&path, json).await?;

    cache
        .update(object_info, crc, region_id, object_type, path)
        .await?;

    Ok(())
}

async fn try_attach_to_avatar(
    cache: &ObjectCache,
    parent_id: u32,
    update: &CachedObjectUpdateType,
    avatars: &HashMap<Uuid, Avatar>,
) -> Option<AvatarAttachment> {
    // Hop 1: Parent is an avatar.
    match cache.get_avatar_agent_id(parent_id).await {
        Ok(agent_id) if avatars.contains_key(&agent_id) => {
            return Some(AvatarAttachment {
                agent_id,
                attachment: AttachmentData {
                    position: update.position(),
                    rotation: update.rotation(),
                    scale: update.scale(),
                    attach_point: update.attach_point(),
                },
            });
        }
        _ => {}
    }

    // Hop 2: Grandparent is an avatar.
    let grandparent_id = match cache.get_parent(parent_id).await {
        Ok(Some(id)) => id,
        _ => return None,
    };

    let agent_id = match cache.get_avatar_agent_id(grandparent_id).await {
        Ok(id) if avatars.contains_key(&id) => id,
        _ => return None,
    };

    let attach_point = match cache.get_update(parent_id).await {
        Ok(path) => match tokio::fs::read(&path).await {
            Ok(bytes) => match serde_json::from_slice::<CachedObjectUpdateType>(&bytes) {
                Ok(parent_update) => parent_update.attach_point(),
                Err(_) => update.attach_point(),
            },
            Err(_) => update.attach_point(),
        },
        Err(_) => update.attach_point(),
    };

    Some(AvatarAttachment {
        agent_id,
        attachment: AttachmentData {
            position: update.position(),
            rotation: update.rotation(),
            scale: update.scale(),
            attach_point,
        },
    })
}

#[derive(Debug, Message)]
#[rtype(result = "()")]
pub struct HandleOrphanedObject(pub OrphanedObject);
impl Handler<HandleOrphanedObject> for SessionActor {
    type Result = ();

    fn handle(&mut self, msg: HandleOrphanedObject, ctx: &mut Self::Context) {
        let Some(session) = self.session.as_ref() else {
            return;
        };

        let cache = self.cache.clone();
        let avatars = session.avatars.clone();
        let addr = ctx.address();

        ctx.spawn(
            async move {
                let orphan_id = msg.0.local_id;
                let parent_id = msg.0.parent_id;

                let Ok(update_path) = cache.object.get_update(orphan_id).await else {
                    warn!("Orphan {orphan_id}: failed to retrieve cached update");
                    return;
                };

                let update_bytes = match tokio::fs::read(&update_path).await {
                    Ok(bytes) => bytes,
                    Err(err) => {
                        warn!("Orphan {orphan_id}: failed to read cached update: {err}");
                        return;
                    }
                };

                let update = match serde_json::from_slice::<CachedObjectUpdateType>(&update_bytes) {
                    Ok(update) => update,
                    Err(err) => {
                        warn!("Orphan {orphan_id}: failed to deserialize: {err}");
                        return;
                    }
                };

                let Some(attachment) =
                    try_attach_to_avatar(&cache.object, parent_id, &update, &avatars).await
                else {
                    // TODO: Handle non-avatar orphan.
                    return;
                };

                reschedule_orphan(&addr, orphan_id, update, attachment);
            }
            .into_actor(self),
        );
    }
}

fn reschedule_orphan(
    addr: &Addr<SessionActor>,
    orphan_id: u32,
    update: CachedObjectUpdateType,
    attachment: AvatarAttachment,
) {
    match update {
        CachedObjectUpdateType::Mesh(mut u) => {
            u.avatar_attachment = Some(attachment);
            addr.do_send(MeshObjectUpdate(u));
        }
        CachedObjectUpdateType::Sculpt(mut u) => {
            u.avatar_attachment = Some(attachment);
            addr.do_send(SculptObjectUpdate(u));
        }
        CachedObjectUpdateType::Prim(mut u) => {
            u.avatar_attachment = Some(attachment);
            addr.do_send(ParametricPrimObjectUpdate(u));
        }
    }
    println!("Rescheduled orphan {orphan_id}");
}

// creates something to await on
pub async fn resolve_asset(addr: Addr<SessionActor>, key: AssetKey) -> Result<PathBuf, String> {
    let request = addr
        .send(GetAsset(key))
        .await
        .map_err(|e| e.to_string())?
        .map_err(|e| e.to_string())?;

    match request {
        AssetRequest::Ready(path) => Ok(path),

        AssetRequest::Pending(mut rx) => loop {
            match rx.borrow_and_update().clone() {
                DownloadState::Complete(path) => return Ok(path),

                DownloadState::Failed(error) => {
                    return Err(error);
                }

                DownloadState::Downloading => {}
            }

            if rx.changed().await.is_err() {
                return Err("Asset download channel closed".into());
            }
        },
    }
}

/// the values an assetrequest can return.
/// Assets can either be ready, or pending.
enum AssetRequest {
    /// the path to the asset on disk
    Ready(PathBuf),
    /// the receiver of the download
    Pending(watch::Receiver<DownloadState>),
}

/// Message for retrieving assets from the server
///
/// This orchestrates pending downloads for all asset objects. This is called and awaited by
/// resolve_asset, which is itself called by any object update that requests an object.
/// This is responsible for
/// - checking the disk to see if the asset is already there,
/// - if the object is not there, subscribe to an existing download, or trigger a new one.
///
/// # Causes
/// [`resolve_asset`] function
///
/// # Effects
/// [`DownloadAsset`]
#[derive(Message)]
#[rtype(result = "Result<AssetRequest, String>")]
struct GetAsset(AssetKey);
impl Handler<GetAsset> for SessionActor {
    type Result = ResponseFuture<Result<AssetRequest, String>>;

    fn handle(&mut self, msg: GetAsset, ctx: &mut Self::Context) -> Self::Result {
        if self.permanent_failures.contains(&msg.0) {
            return Box::pin(async { Err("Asset is in permanent failures list".to_string()) });
        }

        let downloads = self.downloads.clone();
        let addr = ctx.address();

        let dirs = self.session_context.dirs.clone();
        let asset_key = msg.0.clone();

        Box::pin(async move {
            let asset_path = match asset_key {
                AssetKey::Mesh(mesh_id) => {
                    let object_id = ObjectId::new(mesh_id);

                    dirs.ensure_object_dir(object_id)
                        .map_err(|e| e.to_string())?;

                    dirs.object_mesh_file(object_id)
                }
                AssetKey::Texture(texture_id) => {
                    dirs.ensure_textures_dir()
                        .map_err(|e| DownloadError::FileSystemError { error: e }.to_string())?;

                    dirs.texture_file(texture_id)
                }
                AssetKey::Material(material_id) => {
                    dirs.ensure_materials_dir()
                        .map_err(|e| DownloadError::FileSystemError { error: e }.to_string())?;

                    dirs.material_file(material_id)
                }
            };

            // Already downloaded: return immediately.
            if tokio::fs::try_exists(&asset_path)
                .await
                .map_err(|e| DownloadError::FileSystemError { error: e.into() }.to_string())?
            {
                return Ok(AssetRequest::Ready(asset_path));
            }

            // Subscribe to an existing download or claim a new one.
            let (rx, start_download) = {
                let mut downloads = downloads.lock().await;

                if let Some(tx) = downloads.get(&msg.0) {
                    (tx.subscribe(), false)
                } else {
                    let (tx, rx) = watch::channel(DownloadState::Downloading);

                    downloads.insert(msg.0, tx);

                    (rx, true)
                }
            };

            // Start the download outside the registry lock.
            if start_download {
                addr.do_send(DownloadAsset {
                    key: asset_key,
                    retry_count: 0,
                    asset_path,
                });
            }
            Ok(AssetRequest::Pending(rx))
        })
    }
}

/// This handles the downloading of assets from the server
///
/// If the asset is not cached on the disk, this is what downloads it.
/// This handles the retrying and handling errors.
///
/// # Causes
/// [`GetAsset`]
///
/// # Side Effects
/// [Storage: Filesystem]
#[derive(Message, Clone)]
#[rtype(result = "()")]
struct DownloadAsset {
    /// the UUID and type of the asset to download
    key: AssetKey,
    /// how many retries this download has attempted
    retry_count: u32,
    /// the path where the asset will be written once downloaded
    asset_path: PathBuf,
}
impl AsMut<DownloadAsset> for DownloadAsset {
    fn as_mut(&mut self) -> &mut DownloadAsset {
        self
    }
}
impl RetriableData for DownloadAsset {
    fn retry_count(&self) -> u32 {
        self.retry_count
    }
    fn inc_retry_count(&mut self) {
        self.retry_count += 1
    }
}
impl Handler<DownloadAsset> for SessionActor {
    type Result = ResponseFuture<()>;
    fn handle(&mut self, msg: DownloadAsset, ctx: &mut Self::Context) -> Self::Result {
        let downloads = self.downloads.clone();
        let client = self.http_client.clone();
        let rate_limiter = self.rate_limiter.clone();
        let addr = ctx.address();

        let endpoint = self.session.as_ref().and_then(|session| {
            session
                .capability_urls
                .get(&Capability::ViewerAsset)
                .cloned()
        });
        let http_client = self.http_client.clone();

        Box::pin(async move {
            let Some(endpoint) = endpoint else {
                addr.do_send(RetryMessage {
                    message: msg.clone(),
                    info_message: "ViewerAsset not ready; retrying download".to_string(),
                    network_overloaded: false,
                });
                return;
            };
            let Some(tx) = downloads.lock().await.get(&msg.key).cloned() else {
                return;
            };

            match async {
                match msg.key {
                    AssetKey::Mesh(mesh_id) => {
                        download_mesh(
                            mesh_id,
                            &endpoint,
                            &msg.asset_path,
                            http_client,
                            rate_limiter,
                        )
                        .await?;
                    }
                    AssetKey::Texture(texture_id) => {
                        download_texture(
                            texture_id,
                            &endpoint,
                            &msg.asset_path,
                            client,
                            rate_limiter,
                        )
                        .await?;
                    }
                    AssetKey::Material(material_id) => {
                        // Download and cache the raw LLSD material.
                        download_material(
                            material_id,
                            &endpoint,
                            &msg.asset_path,
                            client.clone(),
                            rate_limiter.clone(),
                        )
                        .await?;

                        // Material objects can contain their own textures which need to be
                        // downloaded seperately.
                        // Read the material, and await its texture downloads before continuing
                        let bytes = tokio::fs::read(&msg.asset_path).await?;
                        let material_data = serde_llsd_benthic::auto_from_bytes(&bytes)?;
                        let material = PBRMaterial::from_llsd(material_data)?;

                        let texture_ids: HashSet<Uuid> = material
                            .image_uris()
                            .filter_map(|uri| Uuid::parse_str(uri).ok())
                            .collect();

                        let futures = texture_ids.into_iter().map(|texture_id| {
                            let addr = addr.clone();

                            async move { resolve_asset(addr, AssetKey::Texture(texture_id)).await }
                        });

                        for result in join_all(futures).await {
                            result.map_err(DownloadError::DependencyFailure)?;
                        }
                    }
                }
                Ok::<PathBuf, DownloadError>(msg.asset_path.clone())
            }
            .await
            {
                Ok(path) => {
                    // if it has a path, then the download is complete
                    tx.send_replace(DownloadState::Complete(path));
                    downloads.lock().await.remove(&msg.key);
                }
                Err(
                    e @ DownloadError::Retryable {
                        network_overloaded, ..
                    },
                ) => {
                    // if the error is retryable, re-trigger this download
                    addr.do_send(RetryMessage {
                        message: msg.clone(),
                        info_message: format!("download failed: {e}. Retrying..."),
                        network_overloaded,
                    });
                }
                // if the error is not retryable, set the download state to failed
                Err(e) => {
                    error!("{} download failed permanently: {e}", msg.key);
                    addr.do_send(RegisterPermanentFailure(msg.key.clone()));

                    tx.send_replace(DownloadState::Failed(e.to_string()));

                    downloads.lock().await.remove(&msg.key);
                }
            }
        })
    }
}

/// Simple wrapper function to mark an object as permanently failed
///
/// This prevents re-requests of objects that will never resolve, such as 404s.
/// If a permanent download failure is recorded, this is triggered to add it to the list of failures
/// that should not be reattempted.
///
/// # Causes
/// [`DownloadAsset`]
#[derive(Message, Clone)]
#[rtype(result = "()")]
struct RegisterPermanentFailure(AssetKey);
impl Handler<RegisterPermanentFailure> for SessionActor {
    type Result = ();
    fn handle(&mut self, msg: RegisterPermanentFailure, _ctx: &mut Self::Context) -> Self::Result {
        self.permanent_failures.insert(msg.0);
    }
}

async fn face_textures_or_fallback(
    addr: &Addr<SessionActor>,
    textures: &Option<TextureEntries>,
) -> HashMap<u32, PathBuf> {
    let Some(textures) = textures else {
        return HashMap::from([(u32::MAX, default_texture_path())]);
    };

    let mut entries = Vec::with_capacity(textures.faces.len() + 1);

    // Default texture.
    entries.push((u32::MAX, textures.default.texture_id));

    // Per-face overrides.
    entries.extend(
        textures
            .faces
            .iter()
            .map(|(&face, entry)| (face, entry.texture_id)),
    );

    let futures = entries.into_iter().map(|(face, texture_id)| {
        let addr = addr.clone();

        async move {
            let result = resolve_asset(addr, AssetKey::Texture(texture_id)).await;
            (face, result)
        }
    });

    let results = join_all(futures).await;
    let mut paths = HashMap::new();

    for (face, result) in results {
        match result {
            Ok(path) => {
                paths.insert(face, path);
            }
            Err(e) => {
                paths.insert(face, default_texture_path());
            }
        }
    }

    paths
}

async fn face_pbr_materials(
    addr: &Addr<SessionActor>,
    materials: &Option<MaterialsData>,
) -> HashMap<u32, PathBuf> {
    let Some(materials) = materials else {
        return HashMap::new();
    };
    let futures = materials.faces.iter().map(|(face, material_id)| {
        let addr = addr.clone();
        let face = *face;
        let material_id = *material_id;

        async move {
            let result = resolve_asset(addr, AssetKey::Material(material_id)).await;
            (face as u32, result)
        }
    });

    let results = join_all(futures).await;
    let mut paths = HashMap::new();

    for (face, result) in results {
        match result {
            Ok(path) => {
                paths.insert(face, path);
            }
            Err(e) => {
                error!("Texture download failed for face {face}: {e}");
                paths.insert(face, default_texture_path());
            }
        }
    }

    paths
}

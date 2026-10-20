use super::session::Mailbox;
use crate::COORDINATE_CONVERSION;
use crate::session::OutgoingPacket;
use crate::session::RetryMessage;
use crate::session::SendUIMessage;
use actix::Addr;
use actix::AsyncContext;
use actix::WrapFuture;
use actix::{Handler, Message};
use benthic_default_asset_converter::default_texture_path;
use benthic_default_asset_converter::no_geometry_mesh_path;
use benthic_protocol::messages::ui_protocol::UIMessage;
use benthic_protocol::messages::world::mesh_update::MeshType;
use benthic_protocol::messages::world::mesh_update::MeshUpdate;
use benthic_protocol::objects::objects::AttachmentObjectData;
use benthic_protocol::objects::objects::ObjectInstanceInfo;
use benthic_protocol::objects::objects::ParametricPrimData;
use benthic_protocol::objects::objects::RetriableData;
use benthic_protocol::objects::objects::SculptObjectData;
use benthic_protocol::session::session::create_sub_object_dir;
use benthic_protocol::session::session::create_textures_dir;
use benthic_protocol::session::types::DownloadState;
use log::{error, info};
use metaverse_messages::http::capabilities::Capability;
use metaverse_messages::packet::packet_protocol::Packet;
use metaverse_messages::udp::object::improved_terse_object_update::ImprovedTerseObjectUpdate;
use metaverse_messages::udp::object::object_update::SculptData;
use metaverse_messages::udp::object::object_update_cached::ObjectUpdateCached;
use metaverse_messages::udp::object::request_multiple_objects::RequestMultipleObjects;
use metaverse_messages::utils::object_types::ObjectType;
use metaverse_messages::utils::path::PrimPath;
use metaverse_messages::utils::texture_entry::TextureEntries;
use metaverse_objects::download_asset::download_asset;
use metaverse_objects::download_asset::download_face_textures;
use metaverse_objects::download_asset::download_texture;
use metaverse_objects::errors::DownloadError;
use metaverse_objects::errors::ObjectUpdateError;
use metaverse_objects::mesh::handle_mesh_object;
use metaverse_objects::mesh::mesh_from_json;
use metaverse_objects::object_updates::MeshObjectData;
use metaverse_objects::object_updates::ObjectUpdateAction;
use metaverse_objects::object_updates::RenderMeshData;
use metaverse_objects::object_updates::RenderObjectJSONData;
use metaverse_objects::object_updates::mesh_update;
use metaverse_objects::object_updates::object_update_cached;
use metaverse_objects::parametric_prim::handler::handle_parametric_prim;
use metaverse_objects::sculpt_object::handle_sculpt_object;
use std::collections::HashMap;
use std::path::Path;
use std::path::PathBuf;
use uuid::Uuid;

/// Message for handling object updates of type mesh
///
/// This handles object update information relevant only to mesh objects.
///
/// # Cause
/// - Object update with type Mesh comes in from the udp packet reader
///
/// # Effect
/// If the object is not currently downloaded
/// - [`DownloadMeshObject`]
///
/// If the object exists on disk
/// - [`RenderObjectMessage`]

#[derive(Message)]
#[rtype(result = "()")]
pub struct MeshObjectUpdate(pub MeshObjectData);
impl Handler<MeshObjectUpdate> for Mailbox {
    type Result = ();
    fn handle(&mut self, msg: MeshObjectUpdate, ctx: &mut Self::Context) -> Self::Result {
        let Some(session) = self.session.as_mut() else {
            return;
        };

        let cache = session.cache.clone();
        let addr = ctx.address();
        let out_dir = session.share_dir_root.clone();
        let downloads = self.downloads.clone();
        ctx.spawn(
            async move {
                match mesh_update(cache, msg.0, out_dir, downloads).await {
                    Ok(actions) => {
                        for action in actions {
                            match action {
                                ObjectUpdateAction::DownloadMesh(data) => {
                                    addr.do_send(DownloadMeshObject(data));
                                }
                                ObjectUpdateAction::Render(data) => {
                                    addr.do_send(RenderObjectMessage(data));
                                }
                                _ => {}
                            }
                        }
                    }
                    Err(e) => {
                        error!("HandleObjectUpdate error: {:?}", e)
                    }
                }
            }
            .into_actor(self),
        );
    }
}

/// Message for downloading object update from its capability endpoint
///
/// This downloads the object data, writes the object to disk as json, triggers the metaverse-mesh
/// library to generate its finalized file, and then triggers a MeshUpdate
///  
/// # Cause
/// - [`MeshObjectUpdate`]
///
/// # Effect
/// If the downloads fail and the failure is recoverable
/// [`RetryMessage`]
///
/// If the mesh generation was successful
/// [`RenderObjectMessage`]
///
/// If the mesh generation permanently failed
/// [`RenderBlankMeshMessage`]
#[derive(Debug, Message, Clone)]
#[rtype(result = "()")]
pub struct DownloadMeshObject(MeshObjectData);
impl AsMut<MeshObjectData> for DownloadMeshObject {
    fn as_mut(&mut self) -> &mut MeshObjectData {
        &mut self.0
    }
}
impl RetriableData for DownloadMeshObject {
    fn retry_count(&self) -> u32 {
        self.0.retry_count
    }
    fn inc_retry_count(&mut self) {
        self.0.retry_count += 1
    }
}
impl Handler<DownloadMeshObject> for Mailbox {
    type Result = ();

    fn handle(&mut self, mut msg: DownloadMeshObject, ctx: &mut Self::Context) {
        let server_endpoint = self.session.as_mut().and_then(|session| {
            session
                .capability_urls
                .get(&Capability::ViewerAsset)
                .cloned()
        });
        let Some(server_endpoint) = server_endpoint else {
            self.queue_for_capability(Capability::ViewerAsset, msg);
            return;
        };
        let Some(session) = self.session.as_mut() else {
            return;
        };

        let addr = ctx.address();
        let cache = session.cache.clone();
        let out_dir = session.share_dir_root.clone();
        let semaphore = self.max_concurrent_downloads.clone();
        let downloads = self.downloads.clone();

        ctx.spawn(
            async move {
                // Download material textures.
                let Some(texture_paths) = face_textures_or_fallback(
                    &addr,
                    &out_dir,
                    &msg.0.texture.clone(),
                    &server_endpoint,
                    &mut msg,
                    "Parametric prim",
                )
                .await
                else {
                    return;
                };
                // only allow a certain number of concurrent downloads to prevent flooding the server
                let _permit = match semaphore.acquire_owned().await {
                    Ok(permit) => permit,
                    Err(e) => {
                        error!("Failed to acquire download semaphore: {e}");
                        return;
                    }
                };

                // Download the mesh .
                let mesh_bytes = match download_asset(
                    ObjectType::Mesh.to_string(),
                    msg.0.sculpt_id,
                    &server_endpoint,
                )
                .await
                {
                    Ok(bytes) => bytes,
                    Err(e @ DownloadError::Retryable { .. }) => {
                        addr.do_send(RetryMessage {
                            message: msg,
                            info_message: format!(
                                "Mesh download failed: {e}. Retrying object download..."
                            ),
                        });
                        return;
                    }
                    Err(e) => {
                        error!("Mesh download failed: {:?}", e);
                        // remove the mesh from the pending downloads list
                        let mut downloads = downloads.lock().await;
                        if let Some(tx) = downloads.remove(&msg.0.sculpt_id) {
                            let _ = tx.send(DownloadState::Failed);
                        }
                        error!("Mesh download error: {:?}, rendering empty object", e);

                        addr.do_send(RenderBlankMeshMessage {
                            out_dir,
                            asset_id: msg.0.sculpt_id,
                            object: msg.0.object_info,
                        });
                        return;
                    }
                };

                let action =
                    match handle_mesh_object(cache, &msg.0, out_dir, mesh_bytes, texture_paths)
                        .await
                    {
                        Ok(action) => action,
                        Err(e) => {
                            error!("Mesh object processing failed: {:?}", e);
                            let mut downloads = downloads.lock().await;
                            if let Some(tx) = downloads.remove(&msg.0.sculpt_id) {
                                let _ = tx.send(DownloadState::Failed);
                            }
                            return;
                        }
                    };

                if let ObjectUpdateAction::Render(action) = action {
                    addr.do_send(RenderObjectMessage(action));
                }
            }
            .into_actor(self),
        );
    }
}

/// Message for handling object updates of type parametric prim
///
/// This handles object update information relevant only to parametric prim objects.
/// This begins the process of generating mesh from path information sent in the packet.
///
/// # Cause
/// - Object update with type parametric prim comes in from the udp packet reader
///
/// # Effect
/// If the downloads fail and the failure is recoverable
/// [`RetryMessage`]
///
/// If the mesh generation was successful
/// [`RenderObjectMessage`]
///
/// If the mesh generation permanently failed
/// [`RenderBlankMeshMessage`]
#[derive(Message, Clone)]
#[rtype(result = "()")]
pub struct ParametricPrimObjectUpdate(pub ParametricPrimData<TextureEntries, PrimPath>);
impl AsMut<ParametricPrimData<TextureEntries, PrimPath>> for ParametricPrimObjectUpdate {
    fn as_mut(&mut self) -> &mut ParametricPrimData<TextureEntries, PrimPath> {
        &mut self.0
    }
}
impl RetriableData for ParametricPrimObjectUpdate {
    fn retry_count(&self) -> u32 {
        self.0.retry_count
    }
    fn inc_retry_count(&mut self) {
        self.0.retry_count += 1
    }
}
impl Handler<ParametricPrimObjectUpdate> for Mailbox {
    type Result = ();
    fn handle(
        &mut self,
        mut msg: ParametricPrimObjectUpdate,
        ctx: &mut Self::Context,
    ) -> Self::Result {
        let server_endpoint = self.session.as_mut().and_then(|session| {
            session
                .capability_urls
                .get(&Capability::ViewerAsset)
                .cloned()
        });

        let Some(server_endpoint) = server_endpoint else {
            self.queue_for_capability(Capability::ViewerAsset, msg);
            return;
        };
        let Some(session) = self.session.as_mut() else {
            return;
        };

        let addr = ctx.address();
        let out_dir = session.share_dir_root.clone();
        ctx.spawn(
            async move {
                // retrieve textures, or requeue this message if the failure was retryable
                let Some(texture_paths) = face_textures_or_fallback(
                    &addr,
                    &out_dir,
                    &msg.0.texture.clone(),
                    &server_endpoint,
                    &mut msg,
                    "Parametric prim",
                )
                .await
                else {
                    // message is already requeued, simply return
                    return;
                };

                let actions = match handle_parametric_prim(
                    &out_dir,
                    &msg.0,
                    texture_paths,
                    msg.0.object_info.full_id,
                )
                .await
                {
                    Ok(actions) => actions,
                    Err(ObjectUpdateError::DownloadError(e @ DownloadError::Retryable { .. })) => {
                        addr.do_send(RetryMessage {
                            message: msg,
                            info_message: format!(
                                "Parametric prim processing failed: {e}. Retrying..."
                            ),
                        });
                        return;
                    }
                    Err(e) => {
                        error!(
                            "Handle parametric prim error: {:?}, rendering empty object",
                            e
                        );

                        addr.do_send(RenderBlankMeshMessage {
                            out_dir,
                            asset_id: msg.0.object_info.full_id,
                            object: msg.0.object_info,
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
            .into_actor(self),
        );
    }
}

/// Message for handling object updates of type sculpt object
///
/// This handles object update information relevant only to sculpt objects.
/// This begins the process of generating mesh from downloaded sculpt texture data.
///
/// # Cause
/// - Object update with type sculpt object comes in from the udp packet reader
///
/// # Effect
/// If the downloads fail and the failure is recoverable
/// [`RetryMessage`]
///
/// If the mesh generation was successful
/// [`RenderObjectMessage`]
///
/// If the mesh generation permanently failed
/// [`RenderBlankMeshMessage`]
#[derive(Debug, Message, Clone)]
#[rtype(result = "()")]
pub struct SculptObjectUpdate(pub SculptObjectData<TextureEntries, SculptData>);
impl AsMut<SculptObjectData<TextureEntries, SculptData>> for SculptObjectUpdate {
    fn as_mut(&mut self) -> &mut SculptObjectData<TextureEntries, SculptData> {
        &mut self.0
    }
}
impl RetriableData for SculptObjectUpdate {
    fn retry_count(&self) -> u32 {
        self.0.retry_count
    }
    fn inc_retry_count(&mut self) {
        self.0.retry_count += 1
    }
}
impl Handler<SculptObjectUpdate> for Mailbox {
    type Result = ();
    fn handle(&mut self, mut msg: SculptObjectUpdate, ctx: &mut Self::Context) -> Self::Result {
        let server_endpoint = self.session.as_mut().and_then(|session| {
            session
                .capability_urls
                .get(&Capability::ViewerAsset)
                .cloned()
        });

        let Some(server_endpoint) = server_endpoint else {
            self.queue_for_capability(Capability::ViewerAsset, msg);
            return;
        };
        let Some(session) = self.session.as_mut() else {
            return;
        };

        let out_dir = session.share_dir_root.clone();
        let sculpt_id = msg.0.data.texture_id;

        let addr = ctx.address();
        let base_dir = match create_sub_object_dir(&out_dir, &sculpt_id.to_string()) {
            Ok(path) => path,
            Err(e) => {
                error!(
                    "Failed to create sub-object directory for sculpt {}: {}",
                    sculpt_id, e
                );
                return;
            }
        };
        ctx.spawn(
            async move {
                let Some(texture_paths) = face_textures_or_fallback(
                    &addr,
                    &out_dir,
                    &msg.0.texture.clone(),
                    &server_endpoint,
                    &mut msg,
                    "Sculpt object",
                )
                .await
                else {
                    return;
                };
                // retrieve sculpt texture from endpoint
                let texture_path = base_dir.join(format!("{:?}.png", sculpt_id));
                if let Err(e) = download_texture(sculpt_id, &server_endpoint, &texture_path).await {
                    match e {
                        e @ DownloadError::Retryable { .. } => {
                            addr.do_send(RetryMessage {
                                message: msg,
                                info_message: format!(
                                    "Sculpt texture download failed: {e}. Retrying..."
                                ),
                            });
                        }
                        e => {
                            // if the SculpTexture download gets an unrecoverable error, render an empty
                            // mesh and return early.
                            error!("Sculpt texture  error: {:?}, rendering empty object", e);

                            addr.do_send(RenderBlankMeshMessage {
                                out_dir,
                                asset_id: sculpt_id,
                                object: msg.0.object_info,
                            });
                            return;
                        }
                    }
                    return;
                }

                let sculpt_path = texture_path;

                // build the sculpt object
                let actions = match handle_sculpt_object(
                    &out_dir,
                    &sculpt_path,
                    &msg.0,
                    texture_paths,
                    sculpt_id,
                )
                .await
                {
                    Ok(actions) => actions,
                    Err(ObjectUpdateError::DownloadError(e @ DownloadError::Retryable { .. })) => {
                        addr.do_send(RetryMessage {
                            message: msg,
                            info_message: format!(
                                "Sculpt object processing failed: {e}. Retrying..."
                            ),
                        });
                        return;
                    }
                    Err(e) => {
                        error!("Sculpt texture  error: {:?}, rendering empty object", e);
                        addr.do_send(RenderBlankMeshMessage {
                            out_dir,
                            asset_id: sculpt_id,
                            object: msg.0.object_info,
                        });
                        return;
                    }
                };

                // render the generated sculpt object
                for action in actions {
                    if let ObjectUpdateAction::Render(data) = action {
                        addr.do_send(RenderObjectMessage(data));
                    }
                }
            }
            .into_actor(self),
        );
    }
}

/// Message for handling attachment object updates
/// TODO: currently unimplemented
#[derive(Message)]
#[rtype(result = "()")]
pub struct AttachmentObjectUpdate(pub AttachmentObjectData);
impl Handler<AttachmentObjectUpdate> for Mailbox {
    type Result = ();
    fn handle(&mut self, _msg: AttachmentObjectUpdate, _ctx: &mut Self::Context) -> Self::Result {
        let Some(_session) = self.session.as_mut() else {
            return;
        };
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
impl Handler<HandleImprovedTerseObjectUpdate> for Mailbox {
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
impl Handler<HandleObjectUpdateCached> for Mailbox {
    type Result = ();
    fn handle(&mut self, msg: HandleObjectUpdateCached, ctx: &mut Self::Context) {
        let Some(session) = self.session.as_mut() else {
            return;
        };

        let cache = session.cache.clone();
        let addr = ctx.address();
        let region_id = session.region_data.region_id.clone();
        let session_id = session.session_id;
        let agent_id = session.agent_id;
        let out_dir = session.share_dir_root.clone();
        ctx.spawn(
            async move {
                match object_update_cached(
                    msg.object_update_cached.objects,
                    cache,
                    region_id,
                    out_dir,
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
                            addr.do_send(OutgoingPacket {
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
impl Handler<GenerateMeshMessage> for Mailbox {
    type Result = ();
    fn handle(&mut self, msg: GenerateMeshMessage, ctx: &mut Self::Context) -> Self::Result {
        let Some(session) = self.session.as_mut() else {
            return;
        };

        let cache = session.cache.clone();
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

                            if let Some(tx) = downloads.remove(&action.asset_id) {
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
/// - [`DownloadMeshObject`]
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
impl Handler<RenderObjectMessage> for Mailbox {
    type Result = ();

    fn handle(&mut self, msg: RenderObjectMessage, ctx: &mut Self::Context) -> Self::Result {
        let Some(_session) = self.session.as_mut() else {
            return;
        };

        let addr = ctx.address();

        ctx.spawn(
            async move {
                let mut msg = msg.0;

                if let Some(mut download) = msg.download.take() {
                    if download.changed().await.is_err() {
                        return;
                    }

                    match download.borrow().clone() {
                        DownloadState::Complete(path) => {
                            msg.mesh_path = Some(path);
                        }
                        DownloadState::Failed => {
                            error!("Download failed for object {}", msg.object.full_id);
                            return;
                        }
                        DownloadState::Downloading => {
                            return;
                        }
                    }
                }

                let Some(ref mesh_path) = msg.mesh_path else {
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
                addr.do_send(SendUIMessage {
                    ui_message: UIMessage::new_mesh_update(MeshUpdate {
                        position,
                        scale: msg.object.scale,
                        rotation,
                        parent: msg.object.parent_id,
                        scene_id: Some(msg.object.local_id),
                        path: mesh_path.clone(),
                        mesh_type: MeshType::Object,
                        full_id: Some(msg.object.full_id),
                    }),
                });
            }
            .into_actor(self),
        );
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
pub struct RenderBlankMeshMessage {
    /// base dir of the session. usually ~/.local/share/benthic
    pub out_dir: PathBuf,
    /// ID of the object that failed
    pub asset_id: Uuid,
    /// common instance information such as location, scale, etc.
    pub object: ObjectInstanceInfo,
}
impl Handler<RenderBlankMeshMessage> for Mailbox {
    type Result = ();
    fn handle(&mut self, msg: RenderBlankMeshMessage, ctx: &mut Self::Context) -> Self::Result {
        let base_dir = match create_sub_object_dir(&msg.out_dir, "empty") {
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

/// handler for face textures.
/// this also retries failed downloads.
async fn face_textures_or_fallback<D, M>(
    addr: &Addr<Mailbox>,
    out_dir: &Path,
    texture: &Option<TextureEntries>,
    endpoint: &str,
    msg: &mut M,
    label: &'static str,
) -> Option<HashMap<u32, PathBuf>>
where
    M: AsMut<D> + Message<Result = ()> + Send + Clone + 'static,
    D: RetriableData,
    Mailbox: Handler<M> + Handler<RetryMessage<M>>,
    M: Message<Result = ()> + Send + 'static,
{
    let textures_dir = match create_textures_dir(out_dir) {
        Ok(p) => p,
        Err(_) => {
            error!("Failed to create textures dir");
            return None;
        }
    };

    match download_face_textures(&textures_dir, texture, endpoint).await {
        Ok(paths) => Some(paths),
        Err(e @ DownloadError::Retryable { .. }) => {
            msg.as_mut().inc_retry_count();
            addr.do_send(RetryMessage {
                message: msg.clone(),
                info_message: format!("{label} texture download failed: {e}. Retrying..."),
            });
            None
        }
        Err(e) => {
            error!("{label} texture download failed: {e:?}. using default texture");
            let mut paths = HashMap::new();
            paths.insert(u32::MAX, default_texture_path());
            Some(paths)
        }
    }
}

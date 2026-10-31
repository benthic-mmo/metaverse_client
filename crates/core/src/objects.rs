use super::session::Mailbox;
use crate::COORDINATE_CONVERSION;
use crate::session::OutgoingPacket;
use crate::session::RetryMessage;
use crate::session::SendUIMessage;
use actix::Addr;
use actix::AsyncContext;
use actix::ResponseFuture;
use actix::WrapFuture;
use actix::{Handler, Message};
use benthic_default_asset_converter::default_texture_path;
use benthic_default_asset_converter::no_geometry_mesh_path;
use benthic_protocol::messages::ui_protocol::UIMessage;
use benthic_protocol::messages::world::mesh_update::MeshType;
use benthic_protocol::messages::world::mesh_update::MeshUpdate;
use benthic_protocol::objects::types::AttachmentObjectData;
use benthic_protocol::objects::types::ObjectInstanceInfo;
use benthic_protocol::objects::types::ParametricPrimData;
use benthic_protocol::objects::types::RetriableData;
use benthic_protocol::objects::types::SculptObjectData;
use benthic_protocol::session::storage::create_sub_object_dir;
use benthic_protocol::session::storage::create_textures_dir;
use benthic_protocol::session::types::AssetKey;
use benthic_protocol::session::types::DownloadState;
use futures::future::join_all;
use log::{error, info};
use metaverse_messages::http::capabilities::Capability;
use metaverse_messages::packet::packet_protocol::Packet;
use metaverse_messages::udp::object::improved_terse_object_update::ImprovedTerseObjectUpdate;
use metaverse_messages::udp::object::object_update::SculptData;
use metaverse_messages::udp::object::object_update_cached::ObjectUpdateCached;
use metaverse_messages::udp::object::request_multiple_objects::RequestMultipleObjects;
use metaverse_messages::utils::path::PrimPath;
use metaverse_messages::utils::texture_entry::TextureEntries;
use metaverse_objects::download_asset::download_mesh;
use metaverse_objects::download_asset::download_texture;
use metaverse_objects::errors::DownloadError;
use metaverse_objects::mesh::handle_mesh_object;
use metaverse_objects::mesh::mesh_from_json;
use metaverse_objects::object_updates::MeshObjectData;
use metaverse_objects::object_updates::ObjectUpdateAction;
use metaverse_objects::object_updates::RenderMeshData;
use metaverse_objects::object_updates::RenderObjectJSONData;
use metaverse_objects::object_updates::object_update_cached;
use metaverse_objects::parametric_prim::handler::handle_parametric_prim;
use metaverse_objects::sculpt_object::handle_sculpt_object;
use std::collections::HashMap;
use std::path::PathBuf;
use tokio::sync::watch;
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

#[derive(Message, Clone)]
#[rtype(result = "()")]
pub struct MeshObjectUpdate(pub MeshObjectData);
impl Handler<MeshObjectUpdate> for Mailbox {
    type Result = ();

    fn handle(&mut self, msg: MeshObjectUpdate, ctx: &mut Self::Context) {
        let Some(session) = self.session.as_ref() else {
            return;
        };

        let cache = self.cache.clone();
        let out_dir = session.share_dir_root.clone();
        let addr = ctx.address();

        ctx.spawn(
            async move {
                // Request the mesh and face textures concurrently.
                let mesh_future = resolve_asset(addr.clone(), AssetKey::Mesh(msg.0.sculpt_id));

                let texture_future = face_textures_or_fallback(&addr, &msg.0.texture);

                let (mesh_result, texture_paths) = tokio::join!(mesh_future, texture_future,);

                // Resolve the mesh path.
                let mesh_path = match mesh_result {
                    Ok(path) => path,
                    Err(e) => {
                        error!("Mesh download failed: {e}");

                        addr.do_send(RenderBlankMeshMessage {
                            out_dir,
                            asset_id: msg.0.sculpt_id,
                            object: msg.0.object_info,
                        });
                        return;
                    }
                };

                // Read the downloaded raw mesh.
                let mesh_bytes = match tokio::fs::read(&mesh_path).await {
                    Ok(bytes) => bytes.into(),
                    Err(e) => {
                        error!("Failed to read mesh {}: {e}", msg.0.sculpt_id);
                        return;
                    }
                };

                // Build the GLTF.
                let action =
                    match handle_mesh_object(cache, &msg.0, out_dir, mesh_bytes, texture_paths)
                        .await
                    {
                        Ok(action) => action,
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
impl Handler<ParametricPrimObjectUpdate> for Mailbox {
    type Result = ();

    fn handle(&mut self, msg: ParametricPrimObjectUpdate, ctx: &mut Self::Context) {
        let Some(session) = self.session.as_ref() else {
            return;
        };

        let out_dir = session.share_dir_root.clone();
        let addr = ctx.address();

        ctx.spawn(
            async move {
                // Resolve all face textures.
                let texture_paths = face_textures_or_fallback(&addr, &msg.0.texture).await;

                // Build the parametric prim.
                let actions = match handle_parametric_prim(
                    &out_dir,
                    &msg.0,
                    texture_paths,
                    msg.0.object_info.full_id,
                )
                .await
                {
                    Ok(actions) => actions,
                    Err(e) => {
                        error!("Parametric prim processing failed: {e:?}");

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
impl Handler<SculptObjectUpdate> for Mailbox {
    type Result = ();

    fn handle(&mut self, msg: SculptObjectUpdate, ctx: &mut Self::Context) {
        let Some(session) = self.session.as_ref() else {
            return;
        };

        let out_dir = session.share_dir_root.clone();
        let addr = ctx.address();

        ctx.spawn(
            async move {
                let sculpt_id = msg.0.data.texture_id;

                // Request the sculpt texture and face textures concurrently.
                let sculpt_future = resolve_asset(addr.clone(), AssetKey::Texture(sculpt_id));

                let face_textures_future = face_textures_or_fallback(&addr, &msg.0.texture);

                let (sculpt_result, texture_paths) =
                    tokio::join!(sculpt_future, face_textures_future,);

                // Resolve the sculpt texture.
                let sculpt_path = match sculpt_result {
                    Ok(path) => path,
                    Err(e) => {
                        error!("Sculpt texture download failed: {e}");

                        addr.do_send(RenderBlankMeshMessage {
                            out_dir,
                            asset_id: sculpt_id,
                            object: msg.0.object_info,
                        });

                        return;
                    }
                };

                // Build the sculpt object.
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
                    Err(e) => {
                        error!("Sculpt object processing failed: {e:?}");

                        addr.do_send(RenderBlankMeshMessage {
                            out_dir,
                            asset_id: sculpt_id,
                            object: msg.0.object_info,
                        });

                        return;
                    }
                };

                // Render the generated sculpt object.
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

        let cache = self.cache.clone();
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
        let msg = msg.0;

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

        ctx.address().do_send(SendUIMessage {
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

// creates something to await on
async fn resolve_asset(addr: Addr<Mailbox>, key: AssetKey) -> Result<PathBuf, String> {
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

                DownloadState::Failed { error } => {
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

pub enum AssetRequest {
    Ready(PathBuf),
    Pending(watch::Receiver<DownloadState>),
}
#[derive(Message)]
#[rtype(result = "Result<AssetRequest, String>")]
pub struct GetAsset(pub AssetKey);
impl Handler<GetAsset> for Mailbox {
    type Result = ResponseFuture<Result<AssetRequest, String>>;

    fn handle(&mut self, msg: GetAsset, ctx: &mut Self::Context) -> Self::Result {
        let downloads = self.downloads.clone();
        let addr = ctx.address();

        let Some(session) = self.session.as_ref() else {
            return Box::pin(async { Err(DownloadError::SenderFailure.to_string()) });
        };
        let out_dir = session.share_dir_root.clone();
        let asset_key = msg.0.clone();

        Box::pin(async move {
            let asset_path = match asset_key {
                AssetKey::Mesh(mesh_id) => {
                    let base_dir = create_sub_object_dir(&out_dir, &mesh_id.to_string())
                        .map_err(|e| e.to_string())?;
                    let glb_path = base_dir.join(format!("{:?}_high.bin", mesh_id));
                    glb_path
                }
                AssetKey::Texture(texture_id) => {
                    let textures_dir = create_textures_dir(&out_dir)
                        .map_err(|_| DownloadError::SenderFailure.to_string())?;
                    let texture_path = textures_dir.join(format!("{texture_id}.png"));
                    texture_path
                }
                AssetKey::Material(material_id) => PathBuf::new(),
            };

            // Already downloaded: return immediately.
            if tokio::fs::try_exists(&asset_path)
                .await
                .map_err(|_| DownloadError::SenderFailure.to_string())?
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

#[derive(Message, Clone)]
#[rtype(result = "()")]
pub struct DownloadAsset {
    pub key: AssetKey,
    pub retry_count: u32,
    pub asset_path: PathBuf,
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
impl Handler<DownloadAsset> for Mailbox {
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
            let Some(tx) = downloads.lock().await.get(&msg.key).cloned() else {
                // The request is no longer registered.
                return;
            };

            match async {
                let endpoint = endpoint.ok_or(DownloadError::SenderFailure)?;
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
                    AssetKey::Material(_) => {}
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
                        info_message: format!("texture download failed: {e}. Retrying..."),
                        network_overloaded,
                    });
                }
                // if the error is not retryable, set the download state to failed
                Err(e) => {
                    error!("Texture {} download failed permanently: {e}", msg.key);

                    tx.send_replace(DownloadState::Failed {
                        error: e.to_string(),
                    });

                    downloads.lock().await.remove(&msg.key);
                }
            }
        })
    }
}

async fn face_textures_or_fallback(
    addr: &Addr<Mailbox>,
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
                error!("Texture download failed for face {face}: {e}");
                paths.insert(face, default_texture_path());
            }
        }
    }

    paths
}

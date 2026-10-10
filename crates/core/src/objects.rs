use std::collections::HashMap;

use super::session::Mailbox;
#[cfg(feature = "z-up")]
use crate::COORDINATE_CONVERSION;
use crate::session::OutgoingPacket;
use crate::session::RetryMessage;
use crate::session::SendUIMessage;
use actix::AsyncContext;
use actix::WrapFuture;
use actix::{Handler, Message};
use benthic_protocol::messages::ui::mesh_update::MeshType;
use benthic_protocol::messages::ui::mesh_update::MeshUpdate;
use benthic_protocol::messages::ui::ui_messages::UIMessage;
use benthic_protocol::objects::AttachmentObjectData;
use benthic_protocol::objects::MeshObjectData;
use benthic_protocol::objects::ParametricPrimData;
use benthic_protocol::objects::SculptObjectData;
use benthic_protocol::session::DownloadState;
use benthic_protocol::session::create_sub_object_dir;
use log::{error, warn};
use metaverse_messages::http::capabilities::Capability;
use metaverse_messages::http::scene::SculptType;
use metaverse_messages::packet::packet_protocol::Packet;
use metaverse_messages::udp::object::improved_terse_object_update::ImprovedTerseObjectUpdate;
use metaverse_messages::udp::object::object_update::SculptData;
use metaverse_messages::udp::object::object_update_cached::ObjectUpdateCached;
use metaverse_messages::udp::object::request_multiple_objects::RequestMultipleObjects;
use metaverse_messages::utils::object_types::ObjectType;
use metaverse_messages::utils::path::PrimPath;
use metaverse_messages::utils::texture_entry::TextureEntries;
use metaverse_objects::errors::DownloadError;
use metaverse_objects::object_handler::download_mesh_object;
use metaverse_objects::object_handler::handle_texture;
use metaverse_objects::object_handler::mesh_from_json;
use metaverse_objects::object_updates::DownloadMeshObjectData;
use metaverse_objects::object_updates::GenerateMeshData;
use metaverse_objects::object_updates::ObjectUpdateAction;
use metaverse_objects::object_updates::RenderObjectData;
use metaverse_objects::object_updates::mesh_update;
use metaverse_objects::object_updates::object_update_cached;
use metaverse_objects::parametric_prims::handle_parametric_prim;
use metaverse_objects::sculpt_objects::handle_sculpt_object;

#[derive(Message)]
#[rtype(result = "()")]
pub struct MeshObjectUpdate(pub MeshObjectData<TextureEntries, ObjectType>);
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

#[derive(Message)]
#[rtype(result = "()")]
pub struct ParametricPrimObjectUpdate(pub ParametricPrimData<TextureEntries, PrimPath>);
impl Handler<ParametricPrimObjectUpdate> for Mailbox {
    type Result = ();
    fn handle(
        &mut self,
        mut msg: ParametricPrimObjectUpdate,
        ctx: &mut Self::Context,
    ) -> Self::Result {
        let Some(session) = self.session.as_mut() else {
            return;
        };
        let addr = ctx.address();
        let out_dir = session.share_dir_root.clone();
        let server_endpoint = match session.capability_urls.get(&Capability::ViewerAsset) {
            Some(endpoint) => endpoint.to_string(),
            None => {
                msg.0.retry_count += 1;
                ctx.address().do_send(RetryMessage {
                    retries: msg.0.retry_count,
                    message: msg,
                    long_backoff: true,
                    info_message: "ViewerAsset capability not currently enabled. Requeueing..."
                        .to_string(),
                });
                return;
            }
        };

        ctx.spawn(
            async move {
                match handle_parametric_prim(&out_dir, &msg.0, server_endpoint).await {
                    Ok(actions) => {
                        for action in actions {
                            match action {
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

#[derive(Message)]
#[rtype(result = "()")]
pub struct AttachmentObjectUpdate(pub AttachmentObjectData);
impl Handler<AttachmentObjectUpdate> for Mailbox {
    type Result = ();
    fn handle(&mut self, mut msg: AttachmentObjectUpdate, ctx: &mut Self::Context) -> Self::Result {
        let Some(session) = self.session.as_mut() else {
            return;
        };
    }
}

#[derive(Debug, Message)]
#[rtype(result = "()")]
pub struct SculptObjectUpdate(pub SculptObjectData<TextureEntries, SculptData>);
impl Handler<SculptObjectUpdate> for Mailbox {
    type Result = ();
    fn handle(&mut self, mut msg: SculptObjectUpdate, ctx: &mut Self::Context) -> Self::Result {
        let Some(session) = self.session.as_mut() else {
            return;
        };
        let server_endpoint = match session.capability_urls.get(&Capability::ViewerAsset) {
            Some(endpoint) => endpoint.to_string(),
            None => {
                msg.0.retry_count += 1;
                ctx.address().do_send(RetryMessage {
                    retries: msg.0.retry_count,
                    message: msg,
                    long_backoff: true,
                    info_message: "ViewerAsset capability not currently enabled. Requeueing..."
                        .to_string(),
                });
                return;
            }
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
                match handle_texture(base_dir, sculpt_id, server_endpoint).await {
                    Ok(texture_path) => {
                        match handle_sculpt_object(&out_dir, texture_path, msg.0).await {
                            Ok(actions) => {
                                for action in actions {
                                    match action {
                                        ObjectUpdateAction::Render(data) => {
                                            addr.do_send(RenderObjectMessage(data));
                                        }
                                        _ => {}
                                    }
                                }
                            }
                            Err(e) => {
                                error!("HandleSculptObject error: {:?}", e)
                            }
                        };
                    }
                    Err(e) => {
                        if matches!(e, DownloadError::Retryable { .. }) {
                            msg.0.retry_count += 1;
                            addr.do_send(RetryMessage {
                                retries: msg.0.retry_count,
                                message: msg,
                                long_backoff: true,
                                info_message: format!(
                                    "Download error {}. Retrying object download...",
                                    e
                                ),
                            });
                        }
                    }
                };
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
/// - [`HandlePrim`]
///
/// # Effects
/// - Dispatches a [`MeshUpdate`] to inform the UI of a new object
#[derive(Debug, Message)]
#[rtype(result = "()")]
pub struct DownloadMeshObject(DownloadMeshObjectData);
impl Handler<DownloadMeshObject> for Mailbox {
    type Result = ();
    fn handle(&mut self, mut msg: DownloadMeshObject, ctx: &mut Self::Context) -> Self::Result {
        let Some(session) = self.session.as_mut() else {
            return;
        };

        let server_endpoint = match session.capability_urls.get(&Capability::ViewerAsset) {
            Some(endpoint) => endpoint.to_string(),
            None => {
                msg.0.retry_count += 1;
                ctx.address().do_send(RetryMessage {
                    retries: msg.0.retry_count,
                    message: msg,
                    long_backoff: true,
                    info_message: "ViewerAsset capability not currently enabled. Requeueing..."
                        .to_string(),
                });
                return;
            }
        };
        let addr = ctx.address();
        let cache = session.cache.clone();
        let out_dir = session.share_dir_root.clone();
        let semaphore = self.max_concurrent_downloads.clone();
        let downloads = self.downloads.clone();
        ctx.spawn(
            async move {
                // ensure there aren't more than the permitted amount of downloads happening
                // concurrently.
                let _permit = semaphore.acquire_owned().await.unwrap();

                match download_mesh_object(cache, server_endpoint, &msg.0, out_dir).await {
                    Ok(action) => {
                        if let ObjectUpdateAction::GenerateFromJSON(action) = action {
                            addr.do_send(GenerateMeshMessage(action));
                        }
                    }
                    Err(e) => {
                        if matches!(e, DownloadError::Retryable { .. }) {
                            msg.0.retry_count += 1;
                            addr.do_send(RetryMessage {
                                retries: msg.0.retry_count,
                                message: msg,
                                long_backoff: true,
                                info_message: format!(
                                    "Download error {}. Retrying object download...",
                                    e
                                ),
                            });
                        } else {
                            let mut downloads = downloads.lock().await;
                            if let Some(tx) = downloads.remove(&msg.0.object.sculpt_id) {
                                let _ = tx.send(DownloadState::Failed);
                            }
                        }
                    }
                };
            }
            .into_actor(self),
        );
    }
}

/// Message for handing improved terse object update packets
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
        warn!("ImprovedTerseObjectUpdate packet received. Currently unimplemented.")
    }
}

/// Message for handling ObjectUpdateCached packets
///
/// Retrieves the full object data by sending a RequestMultipleObjects packet. When the server
/// receives this packet, it replies with the ObjectUpdateCompressed packets requested
///
/// # Cause
/// - HandleObjectUpdateCached packet received from UDP socket
///
/// # Effect
/// - [`RequestMultipleObjects`] sent to server
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
                                ObjectUpdateAction::GenerateFromJSON(data) => {
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

/// Helper message to generate mesh from stored json
///
/// # Cause
/// [`HandleObjectUpdateCached`]
/// [`DownloadObject`]
///
/// # Effect
/// [`RenderObjectFromFile`]
#[derive(Debug, Message)]
#[rtype(result = "()")]
pub struct GenerateMeshMessage(pub GenerateMeshData);
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

/// Helper message to render objects from stored json files
///
/// # Cause
/// [`HandleObjectUpdateCached`]
/// [`GenerateMeshFromJson`]
///
/// # Effect
/// [`MeshUpdate`]
#[derive(Debug, Message)]
#[rtype(result = "()")]
pub struct RenderObjectMessage(RenderObjectData);
impl Handler<RenderObjectMessage> for Mailbox {
    type Result = ();

    fn handle(&mut self, msg: RenderObjectMessage, ctx: &mut Self::Context) -> Self::Result {
        let Some(session) = self.session.as_mut() else {
            return;
        };
        let addr = ctx.address();
        let cache = session.cache.clone();
        // check to see if there are any children pending this object
        let children = self.pending_children.clone();
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
                            // Shouldn't normally happen unless the sender changes
                            // the state to Downloading again.
                            return;
                        }
                    }
                }

                let Some(ref mesh_path) = msg.mesh_path else {
                    error!("No mesh path available for object {}", msg.object.full_id);
                    return;
                };

                // if it's the root, just render it.
                let parent_id = match msg.object.parent_id {
                    Some(0) | None => {
                        let children = children
                            .lock()
                            .await
                            .remove(&msg.object.local_id)
                            .unwrap_or_default();
                        let position = COORDINATE_CONVERSION.mul_vec3(msg.object.position);
                        let rotation = COORDINATE_CONVERSION * msg.object.rotation;
                        addr.do_send(SendUIMessage {
                            ui_message: UIMessage::new_mesh_update(MeshUpdate {
                                position,
                                scale: msg.object.scale,
                                rotation,
                                parent: msg.object.parent_id,
                                scene_id: Some(msg.object.local_id),
                                path: mesh_path.clone(),
                                mesh_type: MeshType::Object,
                                id: Some(msg.asset_id),
                            }),
                        });

                        // loop through and send any pending children.
                        // if there are no children, this is 0 and it doesn't run.
                        for child in children {
                            addr.do_send(child);
                        }
                        return;
                    }

                    Some(id) => id,
                };

                match cache.object.get_parent(msg.object.local_id).await {
                    Ok(_parent) => {
                        addr.do_send(SendUIMessage {
                            ui_message: UIMessage::new_mesh_update(MeshUpdate {
                                position: msg.object.position,
                                scale: msg.object.scale,
                                rotation: msg.object.rotation,
                                parent: msg.object.parent_id,
                                scene_id: Some(msg.object.local_id),
                                path: mesh_path.clone(),
                                mesh_type: MeshType::Object,
                                id: Some(msg.asset_id),
                            }),
                        });
                    }

                    Err(inventory_error) => {
                        warn!(
                            "Parent {} not ready, queuing child {}: {}",
                            parent_id, msg.object.full_id, inventory_error
                        );

                        addr.do_send(WaitForParent {
                            parent_id,
                            message: RenderObjectMessage(msg),
                        });
                    }
                }
            }
            .into_actor(self),
        );
    }
}

#[derive(Debug, Message)]
#[rtype(result = "()")]
pub struct WaitForParent {
    pub parent_id: u32,
    pub message: RenderObjectMessage,
}
impl Handler<WaitForParent> for Mailbox {
    type Result = ();

    fn handle(&mut self, msg: WaitForParent, ctx: &mut Self::Context) {
        let pending_children = self.pending_children.clone();

        ctx.spawn(
            async move {
                pending_children
                    .lock()
                    .await
                    .entry(msg.parent_id)
                    .or_default()
                    .push(msg.message);
            }
            .into_actor(self),
        );
    }
}

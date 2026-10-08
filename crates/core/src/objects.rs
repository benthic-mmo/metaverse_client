use super::session::Mailbox;
use crate::avatar::AddObjectToAvatar;
use crate::avatar::DownloadAgentAsset;
use crate::avatar::HandleNewAvatar;
use crate::avatar::LoadFromCache;
use crate::avatar::SetOutfitSize;
use crate::session::OutgoingPacket;
use crate::session::RetryMessage;
use crate::session::SendUIMessage;
use actix::AsyncContext;
use actix::WrapFuture;
use actix::{Handler, Message};
use benthic_protocol::messages::ui::camera_position::CameraPosition;
use benthic_protocol::messages::ui::mesh_update::MeshType;
use benthic_protocol::messages::ui::mesh_update::MeshUpdate;
use benthic_protocol::messages::ui::ui_messages::UIMessage;
use benthic_protocol::objects::AttachmentObjectData;
use benthic_protocol::objects::MeshObjectData;
use benthic_protocol::objects::ParametricPrimData;
use benthic_protocol::session::DownloadState;
use benthic_protocol::session::cache_enabled;
use glam::Quat;
use glam::Vec3;
use log::{error, warn};
use metaverse_avatar::avatar::Avatar;
use metaverse_avatar::avatar::OutfitObject;
use metaverse_avatar::avatar_object_handler::AvatarType;
use metaverse_avatar::errors::AvatarError;
use metaverse_messages::http::capabilities::Capability;
use metaverse_messages::packet::packet_protocol::Packet;
use metaverse_messages::udp::object::improved_terse_object_update::ImprovedTerseObjectUpdate;
use metaverse_messages::udp::object::object_update::ExtraParams;
use metaverse_messages::udp::object::object_update_cached::ObjectUpdateCached;
use metaverse_messages::udp::object::request_multiple_objects::RequestMultipleObjects;
use metaverse_messages::utils::object_types::ObjectType;
use metaverse_messages::utils::texture_entry::TextureEntry;
use metaverse_objects::errors::DownloadError;
use metaverse_objects::object_handler::download_mesh_object;
use metaverse_objects::object_handler::mesh_from_json;
use metaverse_objects::object_updates::DownloadMeshObjectData;
use metaverse_objects::object_updates::GenerateMeshData;
use metaverse_objects::object_updates::ObjectUpdateAction;
use metaverse_objects::object_updates::ObjectUpdateAction::DownloadMesh;
use metaverse_objects::object_updates::RenderObjectData;
use metaverse_objects::object_updates::mesh_update;
use metaverse_objects::object_updates::object_update_cached;
use rand::RngExt;
use uuid::Uuid;

#[derive(Message, Clone)]
#[rtype(result = "()")]
pub struct AvatarObjectUpdate {
    pub id: Uuid,
    pub position: Vec3,
    pub retry_count: u32,
}
impl Handler<AvatarObjectUpdate> for Mailbox {
    type Result = ();
    fn handle(&mut self, mut msg: AvatarObjectUpdate, ctx: &mut Self::Context) -> Self::Result {
        let Some(session) = self.session.as_mut() else {
            return;
        };
        if !session.inventory_data.current_outfit_init {
            let info_message =
                format!("Current outfit not initialized. Requeueing avatar download...");
            msg.retry_count += 1;
            ctx.address().do_send(RetryMessage {
                retries: msg.retry_count,
                message: msg,
                info_message,
                long_backoff: true,
            });
            return;
        }
        let avatar = Avatar::new(msg.id, msg.position);
        let addr = ctx.address();
        let position = Vec3::new(msg.position.x, msg.position.z, msg.position.y);
        if msg.id == session.agent_id {
            addr.do_send(SendUIMessage {
                // handle the z-y flip
                // TODO: THIS IS NOT THE RIGHT WAY TO DO THIS
                // YOU NEED TO MULTIPLY BY THE CORRECTION
                // SHIT IS FUCKED
                // DO NOT LEAVE THIS
                ui_message: UIMessage::new_camera_position(CameraPosition { position }),
            });
            let agent_id = session.agent_id;
            let cache = session.cache.clone();
            let inventory = session.inventory.clone();
            ctx.spawn(
                async move {
                    let (avatar, outfit_items) = if cache_enabled() {
                        match cache.avatar.current_outfit(agent_id, inventory).await {
                            Ok(result) => result,
                            Err(e) => {
                                error!("HandleNewAvatar error: {:?}", e);
                                return;
                            }
                        }
                    } else {
                        match inventory.get_current_outfit().await {
                            Ok(outfit_items) => (None, outfit_items),
                            Err(e) => {
                                error!("HandleNewAvatarError {:?}", e);
                                return;
                            }
                        }
                    };

                    if addr
                        .send(SetOutfitSize {
                            agent_id,
                            outfit_size: outfit_items.len(),
                        })
                        .await
                        .is_err()
                    {
                        error!("Failed to set outfit size for {:?}", agent_id);
                        return;
                    }

                    if let Some(avatar) = avatar {
                        addr.do_send(LoadFromCache { avatar });
                        return;
                    }

                    for item in outfit_items {
                        match item.item_type {
                            ObjectType::Object => {
                                addr.do_send(DownloadAgentAsset {
                                    asset_id: item.asset_id,
                                    item_type: item.item_type,
                                    agent_id,
                                });
                            }
                            ObjectType::Bodypart => {
                                addr.do_send(AddObjectToAvatar {
                                    object: OutfitObject::Bodypart,
                                    agent_id,
                                });
                            }
                            ObjectType::Clothing => {
                                addr.do_send(AddObjectToAvatar {
                                    object: OutfitObject::Clothing,
                                    agent_id,
                                });
                            }
                            _ => {
                                addr.do_send(AddObjectToAvatar {
                                    object: OutfitObject::Other,
                                    agent_id,
                                });
                            }
                        }
                    }
                }
                .into_actor(self),
            );
        } else {
            // TODO: HANDLE NON USER UPDATES
        }
    }
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct MeshObjectUpdate(pub MeshObjectData<TextureEntry, ObjectType>);
impl Handler<MeshObjectUpdate> for Mailbox {
    type Result = ();
    fn handle(&mut self, mut msg: MeshObjectUpdate, ctx: &mut Self::Context) -> Self::Result {
        let Some(session) = self.session.as_mut() else {
            return;
        };

        let cache = session.cache.clone();
        let cache = session.cache.clone();
        let addr = ctx.address();
        let out_dir = session.share_dir_root.clone();
        let downloads = session.downloads.clone();
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
pub struct ParametricPrimObjectUpdate(pub ParametricPrimData);
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
        let downloads = session.downloads.clone();
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
        let downloads = session.downloads.clone();
        ctx.spawn(
            async move {
                match mesh_from_json(cache, msg.0).await {
                    Ok(action) => {
                        if let ObjectUpdateAction::Render(action) = action {
                            let Some(mesh_path) = action.mesh_path else {
                                error!("Generated mesh has no mesh path");
                                return;
                            };

                            let mut downloads = downloads.lock().await;

                            if let Some(tx) = downloads.remove(&action.asset_id) {
                                let _ = tx.send(DownloadState::Complete(mesh_path));
                            }
                        }

                        //if let ObjectUpdateAction::Render(action) = action {
                        //    addr.do_send(RenderObjectMessage(action));
                        //}
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

                let parent_id = match msg.object.parent_id {
                    Some(0) | None => {
                        let conversion = Quat::from_rotation_z(std::f32::consts::PI)
                            * Quat::from_rotation_x(std::f32::consts::FRAC_PI_2);

                        let position = conversion.mul_vec3(msg.object.position);
                        let rotation = conversion * msg.object.rotation;
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

                        return;
                    }

                    Some(id) => id,
                };

                match cache.object.get_scale_rotation_position(parent_id).await {
                    Ok((parent_scale, parent_rotation, parent_position)) => {
                        let rotated_offset = parent_rotation.mul_vec3(msg.object.position);

                        //msg.object.scale = msg.object.scale / parent_scale;
                        //msg.object.position = msg.object.position / parent_scale;

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
                        let info_message = format!(
                            "Parent {} not ready, requeuing object {}: {}",
                            parent_id, msg.object.full_id, inventory_error
                        );

                        msg.retry_count += 1;
                        let retries = msg.retry_count;

                        addr.do_send(RetryMessage {
                            message: RenderObjectMessage(msg),
                            long_backoff: false,
                            info_message,
                            retries,
                        });
                    }
                }
            }
            .into_actor(self),
        );
    }
}

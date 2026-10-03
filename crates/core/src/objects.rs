use super::session::Mailbox;
use crate::avatar::HandleNewAvatar;
use crate::session::OutgoingPacket;
use crate::session::RetryMessage;
use crate::session::SendUIMessage;
use actix::AsyncContext;
use actix::WrapFuture;
use actix::{Handler, Message};
use benthic_protocol::messages::ui::mesh_update::MeshType;
use benthic_protocol::messages::ui::mesh_update::MeshUpdate;
use benthic_protocol::messages::ui::ui_messages::UIMessage;
use benthic_protocol::objects::MinimalObjectUpdate;
use log::{error, warn};
use metaverse_messages::http::capabilities::Capability;
use metaverse_messages::packet::packet_protocol::Packet;
use metaverse_messages::udp::object::improved_terse_object_update::ImprovedTerseObjectUpdate;
use metaverse_messages::udp::object::object_update::ExtraParams;
use metaverse_messages::udp::object::object_update_cached::ObjectUpdateCached;
use metaverse_messages::udp::object::request_multiple_objects::RequestMultipleObjects;
use metaverse_messages::utils::object_types::ObjectType;
use metaverse_messages::utils::texture_entry::TextureEntry;
use metaverse_objects::object_handler::download_object;
use metaverse_objects::object_handler::mesh_from_json;
use metaverse_objects::object_updates::DownloadObjectData;
use metaverse_objects::object_updates::GenerateMeshData;
use metaverse_objects::object_updates::ObjectUpdateAction;
use metaverse_objects::object_updates::RenderObjectData;
use metaverse_objects::object_updates::object_update;
use metaverse_objects::object_updates::object_update_cached;

/// Handles received ObjectUpdate packets.
///
/// This message contains a minimal version of the ObjectUpdate packet, and combines the
/// data for [`ObjectUpdate`] and [`ObjectUpdateCompressed`] packets into a single struct.
///
/// # Cause
/// - ObjectUpdate packet received from UDP socket from server  
///
/// # Effects
/// - Dispatches a [`Avatar`] message if the object is an avatar
/// - Dispatches a [`HandleAttachment`] message if the object is an attachment object
/// - Dispatches a [`HandlePrim`] message if the object is a prim
#[derive(Debug, Message, Clone)]
#[rtype(result = "()")]
pub struct HandleObjectUpdate(pub MinimalObjectUpdate<ExtraParams, TextureEntry, ObjectType>);
impl Handler<HandleObjectUpdate> for Mailbox {
    type Result = ();
    fn handle(&mut self, mut msg: HandleObjectUpdate, ctx: &mut Self::Context) -> Self::Result {
        let Some(session) = self.session.as_mut() else {
            return;
        };
        msg.0.region_id = session.region_data.region_id.clone();

        let cache = session.cache.clone();
        let addr = ctx.address();
        let out_dir = session.share_dir_root.clone();
        ctx.spawn(
            async move {
                match object_update(cache, msg.0, out_dir).await {
                    Ok(actions) => {
                        for action in actions {
                            match action {
                                ObjectUpdateAction::Download(data) => {
                                    addr.do_send(DownloadObject(data));
                                }
                                ObjectUpdateAction::Render(data) => {
                                    addr.do_send(RenderObjectMessage(data));
                                }
                                ObjectUpdateAction::NewAvatar(data) => {
                                    addr.do_send(HandleNewAvatar(data));
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
/// - [`HandlePrim`]
///
/// # Effects
/// - Dispatches a [`MeshUpdate`] to inform the UI of a new object
#[derive(Debug, Message)]
#[rtype(result = "()")]
pub struct DownloadObject(DownloadObjectData);
impl Handler<DownloadObject> for Mailbox {
    type Result = ();
    fn handle(&mut self, mut msg: DownloadObject, ctx: &mut Self::Context) -> Self::Result {
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
        ctx.spawn(
            async move {
                match download_object(cache, server_endpoint, msg.0, out_dir).await {
                    Ok(action) => {
                        if let ObjectUpdateAction::GenerateFromJSON(action) = action {
                            addr.do_send(GenerateMeshMessage(action));
                        }
                    }
                    Err(e) => {
                        error!("DownloadObject error: {:?}", e)
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
        ctx.spawn(
            async move {
                match mesh_from_json(cache, msg.0).await {
                    Ok(action) => {
                        if let ObjectUpdateAction::Render(action) = action {
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
        ctx.spawn(
            async move {
                let mut msg = msg.0;
                let parent_id = match msg.object.parent_id {
                    Some(0) | None => {
                        // no parent, render directly
                        addr.do_send(SendUIMessage {
                            ui_message: UIMessage::new_mesh_update(MeshUpdate {
                                position: msg.object.position,
                                scale: msg.object.scale,
                                rotation: msg.object.rotation,
                                parent: msg.object.parent_id,
                                scene_id: Some(msg.object.local_id),
                                path: msg.mesh_path,
                                mesh_type: MeshType::Object,
                                id: None,
                            }),
                        });
                        return;
                    }
                    Some(id) => id,
                };

                match cache.object.get_scale_rotation_position(parent_id).await {
                    Ok((_parent_scale, parent_rotation, parent_position)) => {
                        let rotated_offset = parent_rotation.mul_vec3(msg.object.position);

                        msg.object.position = parent_position + rotated_offset;
                        msg.object.rotation = parent_rotation * msg.object.rotation;

                        addr.do_send(SendUIMessage {
                            ui_message: UIMessage::new_mesh_update(MeshUpdate {
                                position: msg.object.position,
                                scale: msg.object.scale,
                                rotation: msg.object.rotation,
                                parent: msg.object.parent_id,
                                scene_id: Some(msg.object.local_id),
                                path: msg.mesh_path,
                                mesh_type: MeshType::Object,
                                id: None,
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

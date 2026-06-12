use super::session::Mailbox;
use crate::avatar::HandleNewAvatar;
use crate::initialize::create_sub_agent_dir;
use crate::initialize::create_sub_object_dir;
use crate::session::OutgoingPacket;
use crate::session::SendUIMessage;
use crate::transport::http_handler::download_renderable_mesh;
use crate::transport::http_handler::download_texture;
use actix::AsyncContext;
use actix::ResponseFuture;
use actix::WrapFuture;
use actix::{Handler, Message};
use benthic_protocol::messages::ui::mesh_update::MeshType;
use benthic_protocol::messages::ui::mesh_update::MeshUpdate;
use benthic_protocol::messages::ui::ui_messages::UIMessage;
use glam::Quat;
use glam::Vec3;
use log::info;
use log::{error, warn};
use metaverse_agent::avatar::Avatar;
use metaverse_inventory::object_update::sqlite_check_cache;
use metaverse_inventory::object_update::sqlite_get_object_scale_rotation_position;
use metaverse_inventory::object_update::sqlite_get_parent;
use metaverse_inventory::object_update::sqlite_insert_object_update;
use metaverse_inventory::object_update::sqlite_update_object_glb_path;
use metaverse_inventory::object_update::sqlite_update_object_json_path;
use metaverse_inventory::object_update::GeneratorObject;
use metaverse_mesh::mesh::generate::generate_object_mesh;
use metaverse_messages::http::capabilities::Capability;
use metaverse_messages::packet::packet::Packet;
use metaverse_messages::udp::object::improved_terse_object_update::ImprovedTerseObjectUpdate;
use metaverse_messages::udp::object::object_update::AttachItem;
use metaverse_messages::udp::object::object_update::ExtraParams;
use metaverse_messages::udp::object::object_update_cached::ObjectUpdateCached;
use metaverse_messages::udp::object::request_multiple_objects::CacheMissType;
use metaverse_messages::udp::object::request_multiple_objects::RequestMultipleObjects;
use metaverse_messages::utils::object_types::ObjectType;
use metaverse_messages::utils::texture_entry::TextureEntry;
use serde::Serialize;
use std::fs::File;
use std::io;
use std::io::Write;
use std::path::PathBuf;
use uuid::Uuid;

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
pub struct HandleObjectUpdate {
    /// Type of the object. Required for retrieving full data from the capability endpoint
    pub object_type: ObjectType,
    /// The full ID of the object
    pub full_id: Uuid,
    /// The scene local ID of the object
    pub local_id: u32,
    /// The position of the object.
    ///
    /// If the object is a child object, this position is relative to its parent object.
    pub position: Vec3,
    /// The rotation of the object.
    ///
    /// If the object is a child object, this is used to calculate the
    /// position
    pub rotation: Quat,
    /// The scale of the object.
    pub scale: Vec3,
    /// The local ID of the obeject's parent.
    pub parent: Option<u32>,
    /// The scene local ID of the object's parent.
    ///
    /// This is used to determine the scale position and rotation if the object is part of a construction
    pub parent_id: Option<u32>,
    /// The name value of the object.
    ///
    /// This can encode extra data like attachment objects, or the avatar's name
    pub name_value: Option<String>,
    /// Extra parameters.
    ///
    /// Can contain definitions for things like sculpts (which include meshes), flexi data, light, and more.
    pub extra_params: Option<Vec<ExtraParams>>,

    /// Object's texture data
    pub texture: TextureEntry,

    pub crc: u32,
}

/// Begins the pipeline for handling a prim object.
///
/// Prim objects can include both mesh objects, sculpt objects, and primitive geometry objects.
/// # Cause
/// - [`HandleObjectUpdate`]
///
/// # Effects
/// - Dispatches a [`DownloadObject`] message to retrieve full object data from the server
#[derive(Debug, Message)]
#[rtype(result = "()")]
pub struct HandlePrim {
    /// The prim object to handle
    pub object: HandleObjectUpdate,
}

/// Begins the pipeline for handling an attachment object
///
/// # Cause
/// - [`HandleObjectUpdate`]
///
/// TODO: currently a stub with no effects
#[derive(Debug, Message)]
#[rtype(result = "()")]
pub struct HandleAttachment {
    /// The attachment object to handle
    pub object: HandleObjectUpdate,
    /// The attach item data
    pub item: AttachItem,
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
pub struct DownloadObject {
    /// the object data to download
    pub object: HandleObjectUpdate,
    /// The object's asset ID to retrieve from the ViewerAsset endpoint
    pub asset_id: Uuid,
    /// the object's texture ID to retrieve from the ViewerAsset endpoint  
    pub texture_id: Uuid,
    /// the object's location in space
    pub position: Vec3,
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

#[derive(Debug, Message)]
#[rtype(result = "()")]
pub struct GenerateMeshFromJson {
    pub json_path: PathBuf,
    pub base_dir: PathBuf,
    pub asset_id: Uuid,
    pub object: GeneratorObject,
}

#[derive(Debug, Message)]
#[rtype(result = "()")]
pub struct RenderObjectFromFile {
    pub mesh_path: PathBuf,
    pub asset_id: Uuid,
    pub base_dir: PathBuf,
    pub object: GeneratorObject,
    pub retry_count: u32,
}

fn backoff_ms(retry: u32) -> u64 {
    let base = 50;
    let cap = 2000;

    (base * 2u64.pow(retry.min(6))).min(cap)
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

impl Handler<HandleObjectUpdateCached> for Mailbox {
    type Result = ResponseFuture<()>;
    fn handle(&mut self, msg: HandleObjectUpdateCached, ctx: &mut Self::Context) -> Self::Result {
        let session = match self.session.as_ref() {
            Some(session) => session,
            None => return Box::pin(async {}),
        };

        let db_pool = self.inventory_db_connection.clone();
        let addr = ctx.address();
        let region_id = session.region_data.region_id.clone();
        let session_id = session.session_id.clone();
        let agent_id = session.agent_id.clone();

        Box::pin(async move {
            let mut requests = Vec::new();
            for object in &msg.object_update_cached.objects {
                match sqlite_check_cache(&db_pool, object.id, object.crc.clone(), region_id.clone())
                    .await
                {
                    Ok((asset_id, json_path, glb, generator_object)) => {
                        let base_dir = match create_sub_object_dir(&asset_id.to_string()) {
                            Ok(base_dir) => base_dir,
                            Err(e) => {
                                error!("failed to create base dir: {:?}", e);
                                return;
                            }
                        };

                        if let Some(mesh_path) = glb {
                            addr.do_send(RenderObjectFromFile {
                                mesh_path,
                                base_dir,
                                asset_id,
                                object: generator_object,
                                retry_count: 0,
                            })
                        } else {
                            warn!(
                                "Generated mesh file {:?} not found. Generating now.",
                                asset_id
                            );
                            addr.do_send(GenerateMeshFromJson {
                                json_path,
                                base_dir,
                                asset_id,
                                object: generator_object,
                            });
                        }
                    }
                    Err(e) => {
                        info!(
                            "Cache did not contain {}, {} from {}, {:?}",
                            object.id, object.crc, region_id, e
                        );
                        requests.push((CacheMissType::Normal, object.id));
                    }
                }
            }

            if !requests.is_empty() {
                let request = RequestMultipleObjects {
                    session_id,
                    agent_id,
                    requests,
                };

                addr.do_send(OutgoingPacket {
                    packet: Packet::new_request_multiple_objects(request),
                });
            }
        })
    }
}

impl Handler<HandleObjectUpdate> for Mailbox {
    type Result = ResponseFuture<()>;
    fn handle(&mut self, msg: HandleObjectUpdate, ctx: &mut Self::Context) -> Self::Result {
        let db_pool = self.inventory_db_connection.clone();
        let addr = ctx.address();
        let msg_cloned = msg.clone();

        let session = match self.session.as_ref() {
            Some(session) => session,
            None => return Box::pin(async {}),
        };
        let region_id = session.region_data.region_id.clone();
        Box::pin(async move {
            // all object updates first should be added to the db.
            // if they cannot be added, the object should be retried.
            sqlite_insert_object_update(
                &db_pool,
                msg.local_id,
                msg.full_id,
                msg.crc,
                region_id,
                msg.object_type.clone(),
                msg.parent_id,
                msg.position,
                msg.rotation,
                msg.scale,
            )
            .await
            .unwrap_or_else(|e| {
                error!("Object Update Error: {:?}, {:?}", e, msg.full_id);
                //addr.do_send(msg.clone());
                return;
            });

            match msg.object_type {
                ObjectType::Prim => {
                    // if the msg.name_value can be parsed as an attachment, handle it as an
                    // attachment.
                    if let Some(name_value) = msg.name_value.clone() {
                        match AttachItem::parse_attach_item(name_value) {
                            Ok(item) => {
                                addr.do_send(HandleAttachment { object: msg, item });
                            }
                            Err(_) => {
                                // parsing failed, treat as generic
                                addr.do_send(HandlePrim { object: msg });
                            }
                        }
                    } else {
                        // no name_value, treat as generic
                        addr.do_send(HandlePrim { object: msg });
                    }
                }

                ObjectType::Tree
                | ObjectType::Grass
                | ObjectType::Unknown
                | ObjectType::ParticleSystem
                | ObjectType::NewTree => {
                    // TODO: unimplemented
                    warn!("Received unhandled ObjectUpdate type");
                }
                ObjectType::Avatar => {
                    if let Err(e) = create_sub_agent_dir(&msg.full_id.to_string()) {
                        warn!("Failed to create agent dir for {:?}: {:?}", msg.full_id, e);
                    }
                    // create a new avatar object in the session
                    addr.do_send(HandleNewAvatar {
                        avatar: Avatar::new(msg_cloned.full_id, msg_cloned.position),
                    });
                }
                _ => {
                    warn!("Unknown object type");
                }
            }
        })
    }
}

impl Handler<HandlePrim> for Mailbox {
    type Result = ();
    fn handle(&mut self, msg: HandlePrim, ctx: &mut Self::Context) -> Self::Result {
        if let Some(extra_params) = &msg.object.extra_params {
            for param in extra_params {
                match param {
                    ExtraParams::Sculpt(sculpt) => {
                        ctx.address().do_send(DownloadObject {
                            asset_id: sculpt.texture_id,
                            texture_id: Uuid::nil(),
                            object: msg.object.clone(),
                            position: msg.object.position,
                        });
                    }
                    _ => {
                        warn!("Recieved a non sculpt objectupdate. Currently unimplemented")
                    }
                }
            }
        };
    }
}

impl Handler<HandleAttachment> for Mailbox {
    type Result = ResponseFuture<()>;
    fn handle(&mut self, msg: HandleAttachment, _ctx: &mut Self::Context) -> Self::Result {
        let db_pool = self.inventory_db_connection.clone();
        Box::pin(async move {
            let mut current_id = match msg.object.parent_id {
                Some(id) => id,
                None => return,
            };

            let mut visited = std::collections::HashSet::new();

            loop {
                if !visited.insert(current_id) {
                    break;
                }

                let parent = match sqlite_get_parent(&db_pool, current_id).await {
                    Ok(p) => p,
                    Err(_) => return,
                };

                match parent {
                    Some(p) => {
                        current_id = p;
                    }
                    None => break,
                }
            }
        })
    }
}

impl Handler<DownloadObject> for Mailbox {
    type Result = ();
    fn handle(&mut self, msg: DownloadObject, ctx: &mut Self::Context) -> Self::Result {
        if let Some(session) = self.session.as_mut() {
            let server_endpoint = session
                .capability_urls
                .get(&Capability::ViewerAsset)
                .unwrap()
                .to_string();
            let addr = ctx.address();
            let inventory_db = self.inventory_db_connection.clone();
            ctx.spawn(
                async move {
                    let base_dir = match create_sub_object_dir(&msg.asset_id.to_string()) {
                        Ok(base_dir) => base_dir,
                        Err(e) => {
                            error!("failed to create base dir: {:?}", e);
                            return;
                        }
                    };

                    let texture_id = msg.object.texture.texture_id;
                    let texture_path = base_dir.join(format!("{:?}.png", texture_id));
                    let texture_path = match download_texture(
                        ObjectType::Texture.to_string(),
                        texture_id,
                        &server_endpoint,
                        &texture_path,
                    )
                    .await
                    {
                        Ok(_) => texture_path,
                        Err(e) => {
                            error!("Failed to download prim texture: {:?} {:?}", e, texture_id);
                            PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                                .join("assets")
                                .join("textures")
                                .join("benthic_default_texture.png")
                        }
                    };

                    match download_renderable_mesh(
                        msg.asset_id,
                        "name".to_string(),
                        &server_endpoint,
                        &texture_path,
                    )
                    .await
                    {
                        Ok(render_object) => {
                            // write the json
                            let json_path = match write_json(
                                &render_object,
                                msg.asset_id,
                                msg.asset_id.to_string(),
                            ) {
                                Ok(json) => json,
                                Err(e) => {
                                    error!("Failed to write json: {:?}", e);
                                    return;
                                }
                            };

                            sqlite_update_object_json_path(
                                &inventory_db,
                                msg.object.full_id,
                                msg.asset_id,
                                json_path.to_str().unwrap(),
                            )
                            .await
                            .unwrap_or_else(|e| {
                                error!("Object Update Error: {:?}, {:?}", e, msg.object.full_id);
                                return;
                            });
                            addr.do_send(GenerateMeshFromJson {
                                object: GeneratorObject {
                                    full_id: msg.object.full_id,
                                    local_id: msg.object.local_id,
                                    parent_id: msg.object.parent_id,
                                    rotation: msg.object.rotation,
                                    scale: msg.object.scale,
                                    position: msg.object.position,
                                },
                                asset_id: msg.asset_id,
                                base_dir,
                                json_path,
                            });
                        }
                        Err(e) => {
                            error!("{:?}, {:?}", e, msg);
                        }
                    }
                }
                .into_actor(self),
            );
        }
    }
}

impl Handler<GenerateMeshFromJson> for Mailbox {
    type Result = ();
    fn handle(&mut self, msg: GenerateMeshFromJson, ctx: &mut Self::Context) -> Self::Result {
        let inventory_db = self.inventory_db_connection.clone();
        let addr = ctx.address();
        ctx.spawn(
            async move {
                let glb_path = msg.base_dir.join(format!("{:?}_high.glb", msg.asset_id));
                match generate_object_mesh(msg.json_path, glb_path.clone()) {
                    Ok(_) => {
                        info!("Rendering object at: {:?}", msg.asset_id);
                        sqlite_update_object_glb_path(
                            &inventory_db,
                            msg.object.full_id,
                            glb_path.to_str().unwrap(),
                        )
                        .await
                        .unwrap_or_else(|e| {
                            error!("Object Update Error: {:?}, {:?}", e, msg.object.full_id);
                            return;
                        });
                    }
                    Err(e) => warn!("{:?}", e),
                };
                addr.do_send(RenderObjectFromFile {
                    mesh_path: glb_path,
                    base_dir: msg.base_dir,
                    asset_id: msg.asset_id,
                    object: msg.object,
                    retry_count: 0,
                })
            }
            .into_actor(self),
        );
    }
}

impl Handler<RenderObjectFromFile> for Mailbox {
    type Result = ();

    fn handle(&mut self, mut msg: RenderObjectFromFile, ctx: &mut Self::Context) -> Self::Result {
        let inventory_db = self.inventory_db_connection.clone();
        let addr = ctx.address();

        ctx.spawn(
            async move {
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

                match sqlite_get_object_scale_rotation_position(&inventory_db, parent_id).await {
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
                        warn!(
                            "Parent {} not ready, requeuing object {}: {}",
                            parent_id, msg.object.full_id, inventory_error
                        );

                        schedule_retry(&addr, msg, parent_id, &inventory_error.to_string());
                    }
                }
            }
            .into_actor(self),
        );
    }
}

fn schedule_retry(
    addr: &actix::Addr<Mailbox>,
    mut msg: RenderObjectFromFile,
    parent_id: u32,
    reason: &str,
) {
    if msg.retry_count >= 6 {
        error!(
            "Dropping object {} after max retries (parent {}): {}",
            msg.object.full_id, parent_id, reason
        );
        return;
    }

    let delay = backoff_ms(msg.retry_count);

    warn!(
        "Parent {} not ready for {}, retry {} in {}ms",
        parent_id, msg.object.full_id, msg.retry_count, delay
    );

    msg.retry_count += 1;

    let addr = addr.clone();

    actix::spawn(async move {
        actix::clock::sleep(std::time::Duration::from_millis(delay)).await;
        addr.do_send(msg);
    });
}

/// When an object is retrieved in full, the data will be written in serializable json format, to
/// create a cache. The JSON will then be sent to another crate to convert it into a 3d model that
/// can be rendered.
pub fn write_json<T: Serialize>(data: &T, asset_id: Uuid, filename: String) -> io::Result<PathBuf> {
    match create_sub_object_dir(&asset_id.to_string()) {
        Ok(mut object_dir) => match serde_json::to_string(&data) {
            Ok(json) => {
                object_dir.push(format!("{}.json", filename));
                let mut file = File::create(&object_dir).unwrap();
                file.write_all(json.as_bytes()).unwrap();
                Ok(object_dir)
            }
            Err(e) => {
                error!("Failed to serialize scene group {:?}, {:?}", filename, e);
                Err(io::Error::other(e))
            }
        },
        Err(e) => {
            error!(
                "Failed to create agent dir for {:?}. Unable to cache downloaded items.",
                e
            );
            Err(io::Error::other(e))
        }
    }
}

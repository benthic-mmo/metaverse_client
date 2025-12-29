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
use glam::Quat;
use glam::Vec3;
use log::info;
use log::{error, warn};
use metaverse_agent::avatar::Avatar;
use metaverse_inventory::object_update::get_object_scale_rotation_position;
use metaverse_inventory::object_update::get_object_update;
use metaverse_mesh::generate::generate_object_mesh;
use metaverse_messages::http::capabilities::Capability;
use metaverse_messages::packet::message::UIMessage;
use metaverse_messages::packet::packet::Packet;
use metaverse_messages::udp::object::improved_terse_object_update::ImprovedTerseObjectUpdate;
use metaverse_messages::udp::object::object_update::AttachItem;
use metaverse_messages::udp::object::object_update::ExtraParams;
use metaverse_messages::udp::object::object_update_cached::ObjectUpdateCached;
use metaverse_messages::udp::object::request_multiple_objects::CacheMissType;
use metaverse_messages::udp::object::request_multiple_objects::RequestMultipleObjects;
use metaverse_messages::ui::mesh_update::MeshType;
use metaverse_messages::ui::mesh_update::MeshUpdate;
use metaverse_messages::utils::object_types::ObjectType;
use metaverse_messages::utils::texture_entry::TextureEntry;
use serde::Serialize;
use std::fs::File;
use std::io;
use std::io::Write;
use std::path::PathBuf;
use uuid::Uuid;

use metaverse_inventory::object_update::insert_object_update_minimal;

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

    pub texture: TextureEntry,
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
    type Result = ();
    fn handle(&mut self, msg: HandleObjectUpdateCached, ctx: &mut Self::Context) -> Self::Result {
        if let Some(session) = &self.session {
            let mut requests = Vec::new();
            for object in &msg.object_update_cached.objects {
                requests.push((CacheMissType::Normal, object.id));
            }
            let request = RequestMultipleObjects {
                session_id: session.session_id,
                agent_id: session.agent_id,
                requests,
            };

            ctx.address().do_send(OutgoingPacket {
                packet: Packet::new_request_multiple_objects(request),
            });
        }
    }
}

impl Handler<HandleObjectUpdate> for Mailbox {
    type Result = ResponseFuture<()>;
    fn handle(&mut self, msg: HandleObjectUpdate, ctx: &mut Self::Context) -> Self::Result {
        let db_pool = self.inventory_db_connection.clone();
        let addr = ctx.address();
        let msg_cloned = msg.clone();

        if self.session.is_none() {
            return Box::pin(async {});
        };
        Box::pin(async move {
            // all object updates first should be added to the db.
            // if they cannot be added, the object should be retried.

            insert_object_update_minimal(
                &db_pool,
                msg.local_id,
                msg.full_id,
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
                    //
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
                let obj = match get_object_update(&db_pool, current_id).await {
                    Ok(obj) => obj,
                    Err(_) => return,
                };
                if obj.parent_id == 0 {
                    break;
                }
                current_id = obj.parent_id;
            }
        })
    }
}

impl Handler<DownloadObject> for Mailbox {
    type Result = ();
    fn handle(&mut self, mut msg: DownloadObject, ctx: &mut Self::Context) -> Self::Result {
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
                            let glb_path = base_dir.join(format!("{:?}_high.glb", msg.asset_id));
                            match generate_object_mesh(json_path, glb_path.clone()) {
                                Ok(_) => {
                                    info!("Rendering object at: {:?}", msg.asset_id)
                                }
                                Err(e) => warn!("{:?}", e),
                            };

                            // retrieve the parent's trnasforms from the db to determine the
                            // global position of a child object
                            if let Some(parent_id) = msg.object.parent_id {
                                match get_object_scale_rotation_position(&inventory_db, parent_id)
                                    .await
                                {
                                    Ok((_parent_scale, parent_rotation, parent_position)) => {
                                        let rotated_offset =
                                            parent_rotation.mul_vec3(msg.object.position);

                                        msg.object.position = parent_position + rotated_offset;
                                        msg.object.rotation = parent_rotation * msg.object.rotation;
                                    }
                                    Err(e) => {
                                        error!("{:?}", e);
                                        return;
                                    }
                                };
                            }

                            addr.do_send(SendUIMessage {
                                ui_message: UIMessage::new_mesh_update(MeshUpdate {
                                    position: msg.object.position,
                                    scale: msg.object.scale,
                                    rotation: msg.object.rotation,
                                    parent: msg.object.parent,
                                    scene_id: Some(msg.object.local_id),
                                    path: glb_path,
                                    mesh_type: MeshType::Avatar,
                                    id: None,
                                }),
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

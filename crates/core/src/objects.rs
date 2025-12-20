use super::session::Mailbox;
use crate::initialize::create_sub_agent_dir;
use crate::initialize::create_sub_object_dir;
use crate::transport::http_handler::download_renderable_mesh;
use actix::AsyncContext;
#[cfg(any(feature = "agent", feature = "environment"))]
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
use metaverse_inventory::object_update::set_object_transform_by_id;
use metaverse_mesh::generate::generate_object_mesh;
use metaverse_messages::http::capabilities::Capability;
use metaverse_messages::packet::message::UIMessage;
use metaverse_messages::packet::packet::Packet;
use metaverse_messages::udp::object::improved_terse_object_update::ImprovedTerseObjectUpdate;
use metaverse_messages::udp::object::object_update::AttachItem;
use metaverse_messages::udp::object::object_update::ExtraParams;
#[cfg(any(feature = "agent", feature = "environment"))]
use metaverse_messages::udp::object::object_update::ObjectUpdate;
use metaverse_messages::udp::object::object_update_cached::ObjectUpdateCached;
use metaverse_messages::udp::object::object_update_compressed::ObjectUpdateCompressed;
use metaverse_messages::udp::object::request_multiple_objects::CacheMissType;
use metaverse_messages::udp::object::request_multiple_objects::RequestMultipleObjects;
use metaverse_messages::ui::mesh_update::MeshType;
use metaverse_messages::ui::mesh_update::MeshUpdate;
use metaverse_messages::utils::object_types::ObjectType;
use serde::Serialize;
use std::fs::File;
use std::io;
use std::io::Write;
use std::path::PathBuf;
use uuid::Uuid;

use metaverse_inventory::object_update::insert_object_update_minimal;

#[derive(Debug, Message)]
#[rtype(result = "()")]
pub struct HandlePrim {
    pub object: HandleObjectUpdate,
}

#[derive(Debug, Message)]
#[rtype(result = "()")]
pub struct HandleAttachment {
    pub object: HandleObjectUpdate,
    pub item: AttachItem,
}

#[derive(Debug, Message)]
#[rtype(result = "()")]
pub struct DownloadObject {
    pub object: HandleObjectUpdate,
    pub asset_id: Uuid,
    pub texture_id: Uuid,
    pub position: Vec3,
}

#[derive(Debug, Message, Clone)]
#[rtype(result = "()")]
pub struct HandleObjectUpdate {
    pub object_type: ObjectType,
    pub full_id: Uuid,
    pub id: u32,
    pub parent_id: Option<u32>,
    pub name_value: Option<String>,
    pub extra_params: Option<Vec<ExtraParams>>,
    pub position: Vec3,
    pub rotation: Quat,
    pub scale: Vec3,
    pub parent: Option<u32>,
}

impl Handler<ImprovedTerseObjectUpdate> for Mailbox {
    type Result = ();
    fn handle(&mut self, msg: ImprovedTerseObjectUpdate, ctx: &mut Self::Context) -> Self::Result {
        //println!("{:?}", msg);
    }
}

impl Handler<ObjectUpdateCached> for Mailbox {
    type Result = ();
    fn handle(&mut self, msg: ObjectUpdateCached, ctx: &mut Self::Context) -> Self::Result {
        if let Some(session) = &self.session {
            let mut requests = Vec::new();
            for object in &msg.objects {
                requests.push((CacheMissType::Normal, object.id));
            }
            let request = RequestMultipleObjects {
                session_id: session.session_id,
                agent_id: session.agent_id,
                requests,
            };

            ctx.address()
                .do_send(Packet::new_request_multiple_objects(request));
        }
    }
}

#[cfg(any(feature = "agent", feature = "environment"))]
impl Handler<ObjectUpdate> for Mailbox {
    type Result = ();
    fn handle(&mut self, msg: ObjectUpdate, ctx: &mut Self::Context) -> Self::Result {
        ctx.address().do_send(HandleObjectUpdate {
            object_type: msg.pcode,
            full_id: msg.full_id,
            parent_id: Some(msg.parent_id),
            id: msg.id,
            name_value: Some(msg.name_value),
            position: msg.motion_data.position,
            extra_params: msg.extra_params,
            rotation: msg.motion_data.rotation,
            scale: msg.scale,
            parent: Some(msg.parent_id),
        });
    }
}

impl Handler<ObjectUpdateCompressed> for Mailbox {
    type Result = ();
    fn handle(&mut self, msg: ObjectUpdateCompressed, ctx: &mut Self::Context) -> Self::Result {
        for object in msg.object_data {
            ctx.address().do_send(HandleObjectUpdate {
                object_type: object.pcode,
                full_id: object.full_id,
                parent_id: object.parent_id,
                id: object.local_id,
                name_value: object.name_values,
                position: object.position,
                extra_params: object.extra_params,
                rotation: object.rotation,
                scale: object.scale,
                parent: object.parent_id,
            });
        }
    }
}

#[cfg(any(feature = "agent", feature = "environment"))]
impl Handler<HandleObjectUpdate> for Mailbox {
    type Result = ResponseFuture<()>;
    fn handle(&mut self, msg: HandleObjectUpdate, ctx: &mut Self::Context) -> Self::Result {
        let db_pool = self.inventory_db_connection.clone();
        let addr = ctx.address();
        let msg_cloned = msg.clone();

        if self.session.is_none() {
            return Box::pin(async {});
        };
        println!("object update received: {:?}", msg.object_type);
        Box::pin(async move {
            // all object updates first should be added to the db.
            // if they cannot be added, the object should be retried.

            insert_object_update_minimal(
                &db_pool,
                msg.id,
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
                    //#[cfg(feature = "environment")]
                    //println!(
                    //    "Received environment data of type: {:?}: {:?}",
                    //    msg.pcode, msg
                    //);
                }
                ObjectType::Avatar => {
                    #[cfg(feature = "agent")]
                    if let Err(e) = create_sub_agent_dir(&msg.full_id.to_string()) {
                        warn!("Failed to create agent dir for {:?}: {:?}", msg.full_id, e);
                    }
                    // create a new avatar object in the session
                    addr.do_send(Avatar::new(msg_cloned.full_id, msg_cloned.position));
                }
                _ => {
                    println!("Unknown object type");
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
                        println!("Recieved a non sculpt objectupdate")
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

                    if let Some(parent_id) = msg.object.parent_id {
                        match get_object_scale_rotation_position(&inventory_db, parent_id).await {
                            Ok((parent_scale, parent_rotation, parent_position)) => {
                                let scale = msg.object.scale;
                                let rotation = parent_rotation * msg.object.rotation;
                                let position = parent_rotation * (msg.position * parent_scale)
                                    + parent_position;
                                println!(
                                    "child: {:?}, {:?}, {:?}",
                                    msg.object.scale, msg.object.rotation, msg.position
                                );
                                println!(
                                    "parent: {:?}, {:?}, {:?}",
                                    parent_scale, parent_rotation, parent_position
                                );
                                println!("combined: {:?}, {:?}, {:?}", scale, rotation, position);

                                msg.position = position;
                                msg.object.rotation = rotation;
                                msg.object.scale = scale;
                            }
                            Err(e) => {
                                println!("{:?}", e);
                                return;
                            }
                        };
                    };

                    // TODO: download the texture given in the packet. Needs to be added to
                    // the downloadobject
                    //
                    // let texture_id = scene_group.parts[0].shape.texture.texture_id;
                    let texture_path = PathBuf::from("/home/skclark/Downloads/T_thinkpad_a.png");
                    // match download_texture(
                    //     ObjectType::Texture.to_string(),
                    //     texture_id,
                    //     &server_endpoint,
                    //     &texture_path,
                    // )
                    // .await
                    // {
                    //     Ok(_) => {}
                    //     Err(e) => {
                    //         error!("Failed to download texture: {:?}", e)
                    //     }
                    // }
                    //
                    match download_renderable_mesh(
                        msg.asset_id,
                        "name".to_string(),
                        msg.object.scale,
                        msg.object.rotation,
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
                            addr.do_send(UIMessage::new_mesh_update(MeshUpdate {
                                position: msg.position,
                                path: glb_path,
                                mesh_type: MeshType::Avatar,
                                id: None,
                            }));
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

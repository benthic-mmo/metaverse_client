use std::fs::File;
use std::io;
use std::io::Write;
use std::path::PathBuf;

use super::session::Mailbox;
use crate::initialize::create_sub_agent_dir;
use crate::initialize::create_sub_object_dir;
use crate::transport::http_handler::create_object_render_object;
use crate::transport::http_handler::download_mesh;
use actix::AsyncContext;
#[cfg(any(feature = "agent", feature = "environment"))]
use actix::ResponseFuture;
use actix::WrapFuture;
use actix::{Handler, Message};
use glam::Vec3;
use log::info;
use log::{error, warn};
use metaverse_agent::avatar::Avatar;
use metaverse_inventory::object_update::get_object_update;
use metaverse_inventory::object_update::insert_object_update;
use metaverse_mesh::generate::generate_mesh;
use metaverse_mesh::generate::generate_object_mesh;
use metaverse_mesh::generate::generate_skinned_mesh;
use metaverse_messages::http::capabilities::Capability;
use metaverse_messages::packet::message::UIMessage;
use metaverse_messages::udp::object::object_update::AttachItem;
use metaverse_messages::udp::object::object_update::ExtraParams;
#[cfg(any(feature = "agent", feature = "environment"))]
use metaverse_messages::udp::object::object_update::ObjectUpdate;
use metaverse_messages::ui::mesh_update::MeshType;
use metaverse_messages::ui::mesh_update::MeshUpdate;
use metaverse_messages::utils::object_types::ObjectType;
use serde::Serialize;
use uuid::Uuid;

#[derive(Debug, Message)]
#[rtype(result = "()")]
pub struct HandleObject {
    pub object: ObjectUpdate,
}

#[derive(Debug, Message)]
#[rtype(result = "()")]
pub struct HandleAttachment {
    pub object: ObjectUpdate,
    pub item: AttachItem,
}

#[derive(Debug, Message)]
#[rtype(result = "()")]
pub struct DownloadObject {
    pub asset_id: Uuid,
    pub texture_id: Uuid,
    pub position: Vec3,
}

#[cfg(any(feature = "agent", feature = "environment"))]
impl Handler<ObjectUpdate> for Mailbox {
    type Result = ResponseFuture<()>;
    fn handle(&mut self, msg: ObjectUpdate, ctx: &mut Self::Context) -> Self::Result {
        let db_pool = self.inventory_db_connection.clone();
        let addr = ctx.address();
        let msg_cloned = msg.clone();

        if self.session.is_none() {
            return Box::pin(async {});
        };

        Box::pin(async move {
            // all object updates first should be added to the db.
            // if they cannot be added, the object should be retried.
            insert_object_update(&db_pool, &msg)
                .await
                .unwrap_or_else(|e| {
                    error!("Object Update Error: {:?}, {:?}", e, msg.full_id);
                    addr.do_send(msg.clone());
                    return;
                });

            match msg.pcode {
                ObjectType::Prim => {
                    // if the msg.name_value can be parsed as an attachment, handle it as an
                    // attachment.
                    match AttachItem::parse_attach_item(msg.name_value.clone()) {
                        Ok(item) => {
                            println!("attachment: {:?}", msg.name_value);
                            addr.do_send(HandleAttachment { object: msg, item });
                        }
                        // if not, ignore the error and handle it as a generic object.
                        Err(_) => {
                            println!("generic object: {:?}", msg);
                            addr.do_send(HandleObject { object: msg });
                        }
                    };
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
                    addr.do_send(Avatar::new(
                        msg_cloned.full_id,
                        msg_cloned.motion_data.position,
                    ));
                }
                _ => {
                    println!("Unknown object type");
                }
            }
        })
    }
}

impl Handler<HandleObject> for Mailbox {
    type Result = ();
    fn handle(&mut self, msg: HandleObject, ctx: &mut Self::Context) -> Self::Result {
        if let Some(session) = self.session.as_mut() {
            // send a requestmultipleobject packet
            if let Some(extra_params) = msg.object.extra_params {
                for param in extra_params {
                    match param {
                        ExtraParams::Sculpt(sculpt) => {
                            ctx.address().do_send(DownloadObject {
                                asset_id: sculpt.texture_id,
                                texture_id: Uuid::nil(),
                                position: msg.object.motion_data.position,
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
}

impl Handler<HandleAttachment> for Mailbox {
    type Result = ResponseFuture<()>;
    fn handle(&mut self, msg: HandleAttachment, _ctx: &mut Self::Context) -> Self::Result {
        let db_pool = self.inventory_db_connection.clone();
        Box::pin(async move {
            let mut current_id = msg.object.parent_id;
            loop {
                let obj = match get_object_update(&db_pool, current_id).await {
                    Ok(obj) => obj,
                    Err(e) => {
                        error!("Error fetching parent {:?}: {:?}", current_id, e);
                        return;
                    }
                };

                if obj.parent_id == 0 {
                    break;
                    // root object found
                } else {
                    current_id = obj.parent_id;
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
            ctx.spawn(
                async move {
                    let base_dir = match create_sub_object_dir(&msg.asset_id.to_string()) {
                        Ok(base_dir) => base_dir,
                        Err(e) => {
                            error!("failed to create base dir: {:?}", e);
                            return;
                        }
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
                    match create_object_render_object(
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

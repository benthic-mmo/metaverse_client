use super::session::Mailbox;
use crate::initialize::create_sub_agent_dir;
use actix::AsyncContext;
#[cfg(any(feature = "agent", feature = "environment"))]
use actix::ResponseFuture;
use actix::{Handler, Message};
use log::{error, warn};
use metaverse_agent::avatar::Avatar;
use metaverse_inventory::object_update::get_object_update;
use metaverse_inventory::object_update::insert_object_update;
use metaverse_messages::packet::packet::Packet;
use metaverse_messages::udp::object::object_update::AttachItem;
#[cfg(any(feature = "agent", feature = "environment"))]
use metaverse_messages::udp::object::object_update::ObjectUpdate;
use metaverse_messages::udp::object::request_multiple_objects::CacheMissType;
use metaverse_messages::udp::object::request_multiple_objects::RequestMultipleObjects;
use metaverse_messages::utils::object_types::ObjectType;

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

#[cfg(any(feature = "agent", feature = "environment"))]
impl Handler<ObjectUpdate> for Mailbox {
    type Result = ResponseFuture<()>;
    fn handle(&mut self, msg: ObjectUpdate, ctx: &mut Self::Context) -> Self::Result {
        println!("{:?}", msg);

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
                            addr.do_send(HandleAttachment { object: msg, item });
                        }
                        // if not, ignore the error and handle it as a generic object.
                        Err(_) => {
                            addr.do_send(HandleObject { object: msg });
                            return;
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
            println!("{:?}", msg);
            println!("sending a requestmultipleobject packet");
            ctx.address().do_send(Packet::new_request_multiple_objects(
                RequestMultipleObjects {
                    agent_id: session.agent_id,
                    session_id: session.session_id,
                    requests: [(CacheMissType::Normal, msg.object.id)].to_vec(),
                },
            ));
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

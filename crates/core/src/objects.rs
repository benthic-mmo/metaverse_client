use super::agent::DownloadAgentAsset;
use super::session::Mailbox;
use actix::{AsyncContext, Handler, Message};
use glam::Vec3;
use log::warn;
use metaverse_agent::avatar::Avatar;

use metaverse_inventory::agent::get_current_outfit;
use metaverse_messages::http::item::Item;
use metaverse_messages::udp::core::object_update::AttachItem;
use metaverse_messages::utils::item_metadata::ItemMetadata;
use metaverse_messages::{
    http::capabilities::Capability, udp::core::object_update::ObjectUpdate,
    utils::object_types::ObjectType,
};
use std::time::Duration;
use uuid::Uuid;

use crate::initialize::create_sub_agent_dir;

#[derive(Debug, Message)]
#[rtype(result = "()")]
/// Trigger the function that creates the user model, and sends the data to the UI.
pub struct RenderAgent {
    /// The ID of the agent to render
    pub agent_id: Uuid,
    /// all of the items the agent is wearing
    pub outfit: Vec<Item>,
}

#[derive(Debug, Message)]
#[rtype(result = "()")]
/// Retrieve prim data from a capability url  
pub struct DownloadPrim {
    /// The url of the capability url to retrieve data from
    pub url: String,
    /// The metadata of the item to download
    pub item: ItemMetadata,
    /// The agent ID of the avatar
    pub id: Uuid,
    /// The location of the agent in space
    pub position: Vec3,
}

#[cfg(any(feature = "agent", feature = "environment"))]
impl Handler<ObjectUpdate> for Mailbox {
    type Result = ();
    fn handle(&mut self, msg: ObjectUpdate, ctx: &mut Self::Context) -> Self::Result {
        if let Some(session) = self.session.as_mut() {
            if !session.inventory_data.inventory_init {
                warn!("Inventory not yet populated. Queueing object update...");
                ctx.notify_later(msg, Duration::from_secs(1));
                return;
            }

            match msg.pcode {
                ObjectType::Prim => {
                    // this object type can be used to parent objects to bones on models.
                    // TODO: handle this later.
                    let _ = match AttachItem::parse_attach_item(msg.name_value) {
                        Ok(item) => item,
                        Err(_) => {
                            // error!("{:?}", e);
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
                    // if the ID of the object is your agent ID, you are downloading your own
                    // current outfit.
                    if session.agent_id == msg.full_id {
                        let elements =
                            get_current_outfit(&self.inventory_db_connection.lock().unwrap())
                                .unwrap_or_else(|e| panic!("Failed to get current outfit: {}", e));

                        // just count the objects for now.
                        // TODO: fix this. Figure out the proper size.
                        // keep only elements with type Object
                        let object_elements: Vec<_> = elements
                            .into_iter()
                            .filter(|(_, _, _, t)| *t == ObjectType::Object)
                            .collect();

                        //add the agent to the global agent list. This will be used to look up
                        //the position of agents and what they are wearing.
                        {
                            session.agent_list.lock().unwrap().insert(
                                msg.full_id,
                                Avatar::new(
                                    msg.full_id,
                                    msg.motion_data.position,
                                    object_elements.len(),
                                ),
                            );
                        }
                        // create the agent directory that will contain the agent's
                        // files, such as skeleton json, clothing jsons, and rendered 3d files.
                        if let Err(e) = create_sub_agent_dir(&msg.full_id.to_string()) {
                            warn!("Failed to create agent dir for {:?}: {:?}", msg.full_id, e);
                        }

                        // download all of the assets in the inventory
                        for element in object_elements {
                            ctx.address().do_send(DownloadAgentAsset {
                                url: session
                                    .capability_urls
                                    .get(&Capability::ViewerAsset)
                                    .unwrap()
                                    .to_string(),
                                item_name: element.0,
                                item_id: element.1,
                                asset_id: element.2,
                                item_type: element.3,
                                agent_id: msg.full_id,
                                position: msg.motion_data.position,
                            });
                        }
                    } else {
                        println!("HANDLE NON-USER UPDATE")
                    }
                }
                _ => {
                    println!("Unknown object type");
                }
            }
        }
    }
}

use super::agent::DownloadAgentAsset;
use super::session::Mailbox;
use actix::{AsyncContext, Handler, Message};
use log::warn;
use metaverse_agent::avatar::Avatar;
use metaverse_messages::capabilities::item::Item;
use metaverse_messages::{
    capabilities::capabilities::Capability, core::object_update::ObjectUpdate,
    utils::object_types::ObjectType,
};
use std::time::Duration;
use uuid::Uuid;

#[derive(Debug, Message)]
#[rtype(result = "()")]
/// Trigger the function that creates the user model, and sends the data to the UI.
pub struct RenderAgent {
    /// The ID of the agent to render
    pub agent_id: Uuid,
    /// all of the items the agent is wearing
    pub outfit: Vec<Item>,
}

#[cfg(any(feature = "agent", feature = "environment"))]
impl Handler<ObjectUpdate> for Mailbox {
    type Result = ();
    fn handle(&mut self, msg: ObjectUpdate, ctx: &mut Self::Context) -> Self::Result {
        if let Some(session) = self.session.as_mut() {
            match msg.pcode {
                ObjectType::Tree
                | ObjectType::Grass
                | ObjectType::Prim
                | ObjectType::Unknown
                | ObjectType::ParticleSystem
                | ObjectType::NewTree => {
                    //#[cfg(feature = "environment")]
                    //info!("Received environment data");
                }
                ObjectType::Avatar => {
                    #[cfg(feature = "agent")]
                    // if the ID of the object is your agent ID, you are downloading your own
                    // current outfit.
                    if session.agent_id == msg.full_id {
                        if let Some(current_outfit) = session
                            .inventory_data
                            .inventory_tree
                            .as_ref()
                            .and_then(|tree| tree.children.get(&ObjectType::CurrentOutfit))
                        {
                            let elements = current_outfit.folder.items.clone();
                            //add the agent to the agent list
                            {
                                session.agent_list.lock().unwrap().insert(
                                    msg.full_id,
                                    Avatar::new(
                                        msg.full_id,
                                        msg.motion_data.position,
                                        current_outfit.folder.items.len() / 2,
                                    ),
                                );
                            }

                            for element in elements {
                                ctx.address().do_send(DownloadAgentAsset {
                                    url: session
                                        .capability_urls
                                        .get(&Capability::ViewerAsset)
                                        .unwrap()
                                        .to_string(),
                                    item: element,
                                    agent_id: msg.full_id,
                                    position: msg.motion_data.position,
                                });
                            }
                        } else {
                            // If you are downloading your current inventory, and your inventory is
                            // not loaded into memory, wait until the inventory is loaded.
                            // if the inventory never loads, this will wait forever.
                            // TODO: this should probably timeout somehow.
                            warn!("Inventory not yet populated. Queueing object update...");
                            ctx.notify_later(msg, Duration::from_secs(1));
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

use std::time::Duration;

use actix::{AsyncContext, Handler, WrapFuture};
use log::{info, warn};
use metaverse_messages::{
    capabilities::capabilities::Capability, core::object_update::ObjectUpdate,
    utils::object_types::ObjectType,
};

use crate::http_handler::download_asset;

use super::session::{Mailbox, RenderAgent};

#[cfg(any(feature = "agent", feature = "environment"))]
impl Handler<ObjectUpdate> for Mailbox {
    type Result = ();
    fn handle(&mut self, msg: ObjectUpdate, ctx: &mut Self::Context) -> Self::Result {
        match msg.pcode {
            ObjectType::Tree
            | ObjectType::Grass
            | ObjectType::Prim
            | ObjectType::Unknown
            | ObjectType::ParticleSystem
            | ObjectType::NewTree => {
                #[cfg(feature = "environment")]
                info!("Received environment data");
            }
            ObjectType::Avatar | ObjectType::Bodypart | ObjectType::Clothing => {
                #[cfg(feature = "agent")]
                if let Some(session) = &self.session {
                    // if the ID of the object is your agent ID, you are downloading your own
                    // current outfit.
                    if session.agent_id == msg.full_id {
                        if let Some(current_outfit) = session
                            .inventory_data
                            .inventory_tree
                            .as_ref()
                            .and_then(|tree| tree.children.get(&ObjectType::CurrentOutfit))
                        {
                            // This could have been a request item event, but I want to make
                            // sure that the entire directory is loaded and all of the assets
                            // are downloaded before anything starts rendering.
                            //
                            // this is to keep people's clothes on.
                            let address = ctx.address().clone();
                            let url = session
                                .capability_urls
                                .clone()
                                .get(&Capability::ViewerAsset)
                                .unwrap()
                                .clone();
                            let elements = current_outfit.folder.items.clone();
                            let outfit_path = current_outfit.path.clone();
                            ctx.spawn(
                                async move {
                                    let mut items = Vec::new();
                                    for element in elements {
                                        if ObjectType::Link == element.item_type {
                                            continue;
                                        }
                                        match download_asset(
                                            element.item_type,
                                            element.asset_id,
                                            outfit_path.clone(),
                                            &url,
                                        )
                                        .await
                                        {
                                            Ok(item) => {
                                                items.push(item);
                                            }
                                            Err(e) => warn!("Failed to download asset {:?}", e),
                                        }
                                    }
                                    if let Err(e) =
                                        address.send(RenderAgent { outfit: items }).await
                                    {
                                        warn!("Failed to send render agent request {:?}", e)
                                    };
                                }
                                .into_actor(self),
                            );
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
            }
            _ => {
                println!("other value");
            }
        }
    }
}

use actix::{AsyncContext, Handler, Message, WrapFuture};
use log::error;
use log::{info, warn};
use metaverse_messages::ui::mesh_update::{MeshType, MeshUpdate};
use metaverse_messages::ui::ui_events::UiEventTypes;
use metaverse_messages::{
    capabilities::{capabilities::Capability, item_data::ItemData},
    core::object_update::ObjectUpdate,
    utils::object_types::ObjectType,
};
use std::{fs::create_dir_all, time::Duration};

use crate::http_handler::download_asset;

use super::session::UiMessage;
use super::{generate_gltf::generate_high_lod, session::Mailbox};

/// Trigger the function that creates the user model, and sends the data to the UI.
#[derive(Debug, Message)]
#[rtype(result = "()")]
pub struct RenderAgent {
    pub outfit: Vec<ItemData>,
}

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

impl Handler<RenderAgent> for Mailbox {
    type Result = ();
    fn handle(&mut self, msg: RenderAgent, ctx: &mut Self::Context) -> Self::Result {
        for item in msg.outfit {
            if let Some(mesh) = item.mesh {
                if let Some(data_dir) = dirs::data_dir() {
                    let local_share_dir = data_dir.join("benthic");
                    if !local_share_dir.exists() {
                        if let Err(e) = create_dir_all(&local_share_dir) {
                            error!("Failed to create Benthic share dir {:?}", e)
                        };
                        info!("Created Directory: {:?}", local_share_dir);
                    }
                    let agent_dir = local_share_dir.join("agent");
                    if !agent_dir.exists() {
                        if let Err(e) = create_dir_all(&agent_dir) {
                            error!("Failed to create land dir {:?}", e)
                        };
                        info!("Created Directory: {:?}", agent_dir);
                    }
                    match generate_high_lod(&mesh, agent_dir, "asdf".to_string()) {
                        Ok(path) => ctx.address().do_send(UiMessage::new(
                            UiEventTypes::MeshUpdate,
                            MeshUpdate {
                                position: mesh.position.unwrap(),
                                path,
                                mesh_type: MeshType::Avatar,
                            }
                            .to_bytes(),
                        )),
                        Err(e) => {
                            error!("Failed to generate GLTF {:?}", e)
                        }
                    };
                }
            }
        }
    }
}

use std::{path::PathBuf, time::Duration};

use actix::{AsyncContext, Handler, Message, WrapFuture};
use log::warn;
use metaverse_inventory::inventory_root::{FolderRequest, refresh_inventory};
use metaverse_messages::capabilities::{capabilities::Capability, folder_types::FolderNode};
use uuid::Uuid;

use super::session::Mailbox;

#[cfg(feature = "inventory")]
#[derive(Debug, Message)]
#[rtype(result = "()")]
/// Contains information about the Inventory
pub struct InventoryData {
    /// The root of the inventory, received from the LoginResponse. This is a vector of the base
    /// UUIDs that will be used to create the root of the inventory tree using a
    /// FetchInventoryDescendents2 call.
    pub inventory_root: Option<Vec<Uuid>>,
    /// The root of the inventory lib, received from the LoginResponse. This is a vector of base
    /// UUIDs that will be used to create the root of the inventory lib tree using a
    /// FetchLibDescendents2 call. The library contains the public inventory for the simulator and
    /// is used to retrieve other people's items and appearances.
    pub inventory_lib_root: Option<Vec<Uuid>>,
    /// The UUID of the owner of the inventory lib. Used to create the FetchLibDescendents2 call.
    pub inventory_lib_owner: Option<Vec<Uuid>>,

    /// The in-memory representation of the inventory file tree. Constructed as a tree of Folders.
    pub inventory_tree: Option<FolderNode>,
}

#[derive(Debug, Message)]
#[rtype(result = "()")]
/// Called when the inventory needs to be refreshed. Does a full fetch of the inventory and
/// rebuilds the inventory folders on the disk.
pub struct RefreshInventoryEvent {
    /// The agent ID for the inventory refresh. Determines which endpoint to use. If it's the
    /// current user, fetch the FetchInventoryDescendents2. If it isn't, fetch from the
    /// FetchLibDescendents2 endpoint.
    pub agent_id: Uuid,
}

#[cfg(feature = "inventory")]
impl Handler<RefreshInventoryEvent> for Mailbox {
    type Result = ();
    fn handle(&mut self, msg: RefreshInventoryEvent, ctx: &mut Self::Context) -> Self::Result {
        if let Some(session) = &self.session {
            if session.capability_urls.is_empty() {
                warn!("Capabilities not ready yet. Queueing inventory refresh...");
                ctx.notify_later(msg, Duration::from_secs(1));
            } else {
                let capability_url = if msg.agent_id == session.agent_id {
                    session
                        .capability_urls
                        .get(&Capability::FetchInventoryDescendents2)
                } else {
                    session
                        .capability_urls
                        .get(&Capability::FetchLibDescendents2)
                };
                if let Some(url) = capability_url {
                    // good lord
                    // this is obviously wrong but it works kind of
                    // TODO please god fix this immediately
                    // people are reading this code now
                    let folder_id = session
                        .inventory_data
                        .inventory_root
                        .as_ref()
                        .and_then(|vec| vec.first())
                        .copied()
                        .unwrap_or(Uuid::nil());

                    let owner_id = session.agent_id;
                    let addr = ctx.address();
                    let url = url.clone();
                    ctx.spawn(
                        async move {
                            match refresh_inventory(
                                FolderRequest {
                                    folder_id,
                                    owner_id,
                                    fetch_folders: true,
                                    fetch_items: true,
                                    sort_order: 0,
                                },
                                url,
                                PathBuf::new(),
                            )
                            .await
                            {
                                Ok(inventory_nodes) => {
                                    // set the session's inventory data in memory
                                    addr.do_send(inventory_nodes);
                                }
                                Err(e) => {
                                    println!("REFRESH INVENTORY EVENT {:?}", e)
                                }
                            }
                        }
                        .into_actor(self),
                    );
                }
            }
        }
    }
}

#[cfg(feature = "inventory")]
impl Handler<FolderNode> for Mailbox {
    type Result = ();
    fn handle(&mut self, msg: FolderNode, _: &mut Self::Context) -> Self::Result {
        if let Some(session) = self.session.as_mut() {
            session.inventory_data.inventory_tree = Some(msg);
        }
    }
}

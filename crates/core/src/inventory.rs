use log::error;
use std::time::Duration;

use actix::{AsyncContext, Handler, Message, WrapFuture};
use log::warn;
use metaverse_messages::http::capabilities::Capability;
use metaverse_messages::http::folder_request::FolderRequest;
use uuid::Uuid;

use metaverse_inventory::inventory_root::refresh_inventory;

use super::session::Mailbox;

#[cfg(feature = "inventory")]
#[derive(Debug, Message)]
#[rtype(result = "()")]
/// Contains information about the Inventory
pub struct InventoryData {
    /// The root of the inventory, received from the LoginResponse. This is a vector of the base
    /// UUIDs that will be used to create the root of the inventory tree using a
    /// FetchInventoryDescendents2 call.
    pub inventory_root: Uuid,
    /// The UUID of the owner of the inventory lib. Used to create the FetchLibDescendents2 call.
    pub inventory_lib_owner: Uuid,
    pub inventory_init: bool,
}

/// an empty struct to notify the mailbox the inventory has been initialized
#[derive(Debug, Message)]
#[rtype(result = "()")]
pub struct InventoryInit;

#[derive(Debug, Message)]
#[rtype(result = "()")]
/// Called when the inventory needs to be refreshed. Does a full fetch of the inventory from the
/// root and rebuilds the inventory folders on the disk.
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
                // if the message's agent id is the session ID, use the FetchInventoryDescendents2
                // endpoint.
                let (capability_url, folder_id, owner_id) = if msg.agent_id == session.agent_id {
                    (
                        session
                            .capability_urls
                            .get(&Capability::FetchInventoryDescendents2),
                        session.inventory_data.inventory_root,
                        session.agent_id,
                    )
                } else {
                    (
                        session
                            .capability_urls
                            .get(&Capability::FetchLibDescendents2),
                        Uuid::nil(),
                        session.inventory_data.inventory_lib_owner,
                    )
                };
                if let Some(url) = capability_url {
                    let url = url.clone();
                    let addr = ctx.address();
                    let conn = self.inventory_db_connection.clone();
                    ctx.spawn(
                        async move {
                            let mut conn = conn.lock().unwrap();
                            match refresh_inventory(
                                &mut conn,
                                FolderRequest {
                                    folder_id,
                                    owner_id,
                                    fetch_folders: true,
                                    fetch_items: true,
                                    sort_order: 0,
                                },
                                url,
                            )
                            .await
                            {
                                Ok(_) => {
                                    addr.do_send(InventoryInit);
                                }
                                Err(e) => {
                                    error!("REFRESH INVENTORY EVENT FAILED {:?}", e)
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
impl Handler<InventoryInit> for Mailbox {
    type Result = ();
    fn handle(&mut self, _: InventoryInit, _: &mut Self::Context) -> Self::Result {
        if let Some(session) = &mut self.session {
            session.inventory_data.inventory_init = true;
        }
    }
}

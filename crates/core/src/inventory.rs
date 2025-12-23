use log::error;
use std::time::Duration;

use actix::{AsyncContext, Handler, Message, WrapFuture};
use log::warn;
use metaverse_messages::http::capabilities::Capability;
use metaverse_messages::http::folder_request::FolderRequest;
use uuid::Uuid;

use metaverse_inventory::inventory_root::refresh_inventory;

use super::session::Mailbox;

/// Contains information about the Inventory
#[derive(Debug)]
pub struct InventoryData {
    /// The root of the inventory, received from the LoginResponse. This is a vector of the base
    /// UUIDs that will be used to create the root of the inventory tree using a
    /// FetchInventoryDescendents2 call.
    pub inventory_root: Uuid,
    /// The UUID of the owner of the inventory lib. Used to create the FetchLibDescendents2 call.
    pub inventory_lib_owner: Uuid,
    /// boolean to signify the inventory has successfully loaded and is ready for use.
    pub inventory_init: bool,
}

/// Message to inform the session that the inventory has been fully initialized.
///
/// # Cause
/// - [`RefreshInventoryEvent`] successfully initialized inventory  
#[derive(Debug, Message)]
#[rtype(result = "()")]
pub struct InventoryInit;

/// Performs a full refresh on the user's inventory
///
/// Fetches the inventory from the root received from the login response packet, and stores the
/// folder data in the on-disk db.
///
/// # Cause
/// - handle_login function in session.rs after the UIResponse Login has been received.
///
/// # Effects
/// - Dispatches a [`InventoryInit`] message after initialization
#[derive(Debug, Message)]
#[rtype(result = "()")]
pub struct RefreshInventoryEvent {
    /// The agent ID for the inventory refresh. Determines which endpoint to use.
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
                let capability_url = session
                    .capability_urls
                    .get(&Capability::FetchInventoryDescendents2);

                if let Some(url) = capability_url {
                    let owner_id = session.agent_id;
                    let folder_id = session.inventory_data.inventory_root;
                    let url = url.clone();
                    let addr = ctx.address();
                    let conn = self.inventory_db_connection.clone();
                    ctx.spawn(
                        async move {
                            match refresh_inventory(
                                &conn,
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

impl Handler<InventoryInit> for Mailbox {
    type Result = ();
    fn handle(&mut self, _: InventoryInit, _: &mut Self::Context) -> Self::Result {
        if let Some(session) = &mut self.session {
            session.inventory_data.inventory_init = true;
        }
    }
}

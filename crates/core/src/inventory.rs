use benthic_protocol::messages::ui::ui_messages::UIMessage;
use log::error;
use std::time::Duration;

use crate::session::{RetryMessage, SendUIMessage};

use super::session::Mailbox;
use actix::{AsyncContext, Handler, Message, WrapFuture};
use log::warn;
use metaverse_messages::http::capabilities::Capability;
use metaverse_messages::http::folder_request::FolderRequest;
use uuid::Uuid;

#[derive(Debug, Message)]
#[rtype(result = "()")]
pub struct FetchCurrentOutfit {
    pub retry_count: u32,
}
impl Handler<FetchCurrentOutfit> for Mailbox {
    type Result = ();
    fn handle(&mut self, mut msg: FetchCurrentOutfit, ctx: &mut Self::Context) -> Self::Result {
        let Some(session) = self.session.as_mut() else {
            return;
        };
        if session.capability_urls.is_empty() {
            let info_message =
                "Capabilities not ready yet. Requeueing current outfit fetch ...".to_string();
            msg.retry_count += 1;
            ctx.address().do_send(RetryMessage {
                retries: msg.retry_count,
                message: msg,
                info_message,
                long_backoff: true,
            });
            return;
        }

        let Some(url) = session
            .capability_urls
            .get(&Capability::FetchInventoryDescendents2)
            .cloned()
        else {
            return;
        };

        let current_outfit_root = session.inventory_data.current_outfit_root;
        let addr = ctx.address().clone();
        let owner_id = session.agent_id;
        let inventory = session.inventory.clone();

        ctx.spawn(
            async move {
                match inventory
                    .predownload_current_outfit(current_outfit_root, owner_id, url.clone())
                    .await
                {
                    Ok(_) => {
                        addr.do_send(AvatarInit);
                    }
                    Err(e) => {
                        error!("Refresh inventory event failed {:?}", e)
                    }
                }
            }
            .into_actor(self),
        );
    }
}

/// Message to inform the session that the inventory has been fully initialized.
///
/// # Cause
/// - [`RefreshInventoryEvent`] successfully initialized inventory  
#[derive(Debug, Message)]
#[rtype(result = "()")]
pub struct InventoryInit;
impl Handler<InventoryInit> for Mailbox {
    type Result = ();
    fn handle(&mut self, _: InventoryInit, ctx: &mut Self::Context) -> Self::Result {
        let Some(session) = self.session.as_mut() else {
            return;
        };
        session.inventory_data.inventory_init = true;
        let addr = ctx.address().clone();
        let inventory = session.inventory.clone();
        ctx.spawn(
            async move {
                for folder in match inventory.fetch_inventory().await {
                    Ok(folders) => folders,
                    Err(e) => {
                        error!("InventoryInit {:}", e);
                        return;
                    }
                } {
                    addr.do_send(SendUIMessage {
                        ui_message: UIMessage::PopulateInventory(folder),
                    });
                }
            }
            .into_actor(self),
        );
    }
}

#[derive(Debug, Message)]
#[rtype(result = "()")]
pub struct AvatarInit;
impl Handler<AvatarInit> for Mailbox {
    type Result = ();
    fn handle(&mut self, _: AvatarInit, _: &mut Self::Context) -> Self::Result {
        let Some(session) = self.session.as_mut() else {
            return;
        };
        session.inventory_data.current_outfit_init = true;
    }
}

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
    pub retry_count: u32,
}
#[cfg(feature = "inventory")]
impl Handler<RefreshInventoryEvent> for Mailbox {
    type Result = ();
    fn handle(&mut self, mut msg: RefreshInventoryEvent, ctx: &mut Self::Context) -> Self::Result {
        let Some(session) = self.session.as_mut() else {
            return;
        };

        if session.capability_urls.is_empty() {
            let info_message = format!("Capabilities not ready yet. Queueing inventory refresh...");
            msg.retry_count += 1;
            ctx.address().do_send(RetryMessage {
                retries: msg.retry_count,
                message: msg,
                info_message,
                long_backoff: true,
            });
            return;
        }

        let Some(url) = session
            .capability_urls
            .get(&Capability::FetchInventoryDescendents2)
            .cloned()
        else {
            return;
        };
        let owner_id = session.agent_id;
        let folder_id = session.inventory_data.inventory_root;
        let url = url.clone();
        let addr = ctx.address();
        let inventory = session.inventory.clone();
        ctx.spawn(
            async move {
                match inventory
                    .refresh(
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
                        error!("Refresh inventory event failed {:?}", e)
                    }
                }
            }
            .into_actor(self),
        );
    }
}

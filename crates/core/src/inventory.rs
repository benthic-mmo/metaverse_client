use super::session::Mailbox;
use crate::session::SendUIMessage;
use actix::{AsyncContext, Handler, Message, WrapFuture};
use benthic_protocol::messages::ui_protocol::UIMessage;
use log::error;
use metaverse_messages::http::capabilities::Capability;
use metaverse_messages::http::folder_request::FolderRequest;

/// Message to retrieve the current outfit from the sqlite cache.
///
/// # Cause
/// - sent by session.rs immediately after a successful login
///
/// # Effect
/// - [`AvatarInit`]
#[derive(Debug, Message)]
#[rtype(result = "()")]
pub struct FetchCurrentOutfit {}
impl Handler<FetchCurrentOutfit> for Mailbox {
    type Result = ();
    fn handle(&mut self, msg: FetchCurrentOutfit, ctx: &mut Self::Context) -> Self::Result {
        let server_endpoint = self.session.as_mut().and_then(|session| {
            session
                .capability_urls
                .get(&Capability::FetchInventoryDescendents2)
                .cloned()
        });

        let Some(server_endpoint) = server_endpoint else {
            self.queue_for_capability(Capability::FetchInventoryDescendents2, msg);
            return;
        };

        let Some(session) = self.session.as_mut() else {
            return;
        };

        let current_outfit_root = session.inventory_data.current_outfit_root;
        let addr = ctx.address().clone();
        let owner_id = session.agent_id;
        let inventory = session.inventory.clone();

        ctx.spawn(
            async move {
                match inventory
                    .predownload_current_outfit(
                        current_outfit_root,
                        owner_id,
                        server_endpoint.clone(),
                    )
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

/// Message to inform the session that the avatar's current outfit folder has been fully downloaded.
///
/// # Cause
/// - [`FetchCurrentOutfit`]
#[derive(Debug, Message)]
#[rtype(result = "()")]
pub struct AvatarInit;
impl Handler<AvatarInit> for Mailbox {
    type Result = ();

    fn handle(&mut self, _: AvatarInit, ctx: &mut Self::Context) -> Self::Result {
        let Some(session) = self.session.as_mut() else {
            return;
        };
        session.inventory_data.current_outfit_init = true;
        let pending = std::mem::take(&mut self.pending_avatar_objects);
        for update in pending {
            ctx.address().do_send(update);
        }
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
pub struct RefreshInventoryEvent {}
#[cfg(feature = "inventory")]
impl Handler<RefreshInventoryEvent> for Mailbox {
    type Result = ();
    fn handle(&mut self, msg: RefreshInventoryEvent, ctx: &mut Self::Context) -> Self::Result {
        let server_endpoint = {
            let Some(session) = self.session.as_mut() else {
                return;
            };
            session
                .capability_urls
                .get(&Capability::FetchInventoryDescendents2)
                .cloned()
        };
        let Some(server_endpoint) = server_endpoint else {
            self.queue_for_capability(Capability::FetchInventoryDescendents2, msg);
            return;
        };
        let Some(session) = self.session.as_mut() else {
            return;
        };

        let owner_id = session.agent_id;
        let folder_id = session.inventory_data.inventory_root;
        let url = server_endpoint.clone();
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

use std::collections::HashMap;

use super::session::Mailbox;
use actix::{AsyncContext, Handler, Message, WrapFuture};
use log::error;
use metaverse_messages::capabilities::capabilities::{Capability, CapabilityRequest};

/// Message to update the capability urls
#[derive(Debug, Message)]
#[rtype(result = "()")]
pub struct SetCapabilityUrls {
    capability_urls: HashMap<Capability, String>,
}

impl Handler<SetCapabilityUrls> for Mailbox {
    type Result = ();
    fn handle(&mut self, msg: SetCapabilityUrls, _: &mut Self::Context) -> Self::Result {
        if let Some(session) = &mut self.session {
            session.capability_urls.extend(msg.capability_urls);
        }
    }
}

impl Handler<CapabilityRequest> for Mailbox {
    type Result = ();
    fn handle(&mut self, msg: CapabilityRequest, ctx: &mut Self::Context) -> Self::Result {
        if let Some(session) = &self.session {
            let seed_capability_url = session.seed_capability_url.clone();
            let address = ctx.address().clone();
            ctx.spawn(
                async move {
                    let client = awc::Client::default();
                    match client
                        .post(seed_capability_url)
                        .insert_header(("Content-Type", "application/llsd+xml"))
                        .send_body(msg.capabilities)
                        .await
                    {
                        Ok(mut get) => match get.body().await {
                            Ok(body) => {
                                let capability_urls = CapabilityRequest::response_from_llsd(&body);
                                address.do_send(SetCapabilityUrls { capability_urls });
                            }
                            Err(e) => {
                                error!("Failed to retrieve body of capability request {:?}", e);
                            }
                        },
                        Err(e) => {
                            error!("Failed to send with {:?}", e);
                        }
                    };
                }
                .into_actor(self),
            );
        } else {
        }
    }
}

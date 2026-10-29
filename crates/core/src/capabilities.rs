use super::session::Mailbox;
use crate::errors::CapabilityError;
use actix::{AsyncContext, Handler, Message, WrapFuture};
use awc::Client;
use log::error;
use metaverse_messages::http::capabilities::{Capability, CapabilityRequest};
use std::collections::HashMap;

/// Message to request full capability urls from the esrver
///
/// # Cause
/// - Successful login, from the handle_login function in session.rs
///
/// # Effect
/// - Seed capability URL HTTP post
/// - [`SetCapabilityUrls`] if the post was successful
#[derive(Debug, Message)]
#[rtype(result = "()")]
pub struct SendCapabilityRequest {
    /// The capabilities requested
    pub capability_request: CapabilityRequest,
}
impl Handler<SendCapabilityRequest> for Mailbox {
    type Result = ();
    fn handle(&mut self, msg: SendCapabilityRequest, ctx: &mut Self::Context) -> Self::Result {
        if let Some(session) = &self.session {
            let seed_capability_url = session.seed_capability_url.clone();
            let address = ctx.address().clone();
            let caps = msg.capability_request.capabilities;
            let client = self.http_client.clone();
            ctx.spawn(
                async move {
                    match send_cap_request(seed_capability_url, caps, client).await {
                        Ok(capability_urls) => {
                            address.do_send(SetCapabilityUrls { capability_urls });
                        }
                        Err(e) => {
                            error!("{:?}", e)
                        }
                    }
                }
                .into_actor(self),
            );
        }
    }
}

/// Message to update the capability urls
///
/// # Cause
/// [`SendCapabilityRequest`]
#[derive(Debug, Message)]
#[rtype(result = "()")]
pub struct SetCapabilityUrls {
    capability_urls: HashMap<Capability, String>,
}
impl Handler<SetCapabilityUrls> for Mailbox {
    type Result = ();
    fn handle(&mut self, msg: SetCapabilityUrls, ctx: &mut Self::Context) {
        let capabilities: Vec<Capability> = msg.capability_urls.keys().cloned().collect();

        if let Some(session) = &mut self.session {
            session.capability_urls.extend(msg.capability_urls);
        } else {
            return;
        }

        for capability in capabilities {
            self.flush_capability(capability, ctx);
        }
    }
}

async fn send_cap_request(
    seed_capability_url: String,
    capabilities: String,
    client: Client,
) -> Result<HashMap<Capability, String>, CapabilityError> {
    let body = client
        .post(seed_capability_url)
        .insert_header(("Content-Type", "application/llsd+xml"))
        .send_body(capabilities)
        .await?
        .body()
        .await?;
    Ok(CapabilityRequest::response_from_llsd(&body)?)
}

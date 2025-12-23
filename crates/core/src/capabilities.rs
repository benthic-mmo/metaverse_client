use super::session::Mailbox;
use actix::{AsyncContext, Handler, Message, WrapFuture};
use log::error;
use metaverse_messages::http::capabilities::{Capability, CapabilityRequest};
use std::collections::HashMap;

/// Message to update the capability urls
///
/// # Cause
/// [`SendCapabilityRequest`]
#[derive(Debug, Message)]
#[rtype(result = "()")]
pub struct SetCapabilityUrls {
    capability_urls: HashMap<Capability, String>,
}

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

impl Handler<SetCapabilityUrls> for Mailbox {
    type Result = ();
    fn handle(&mut self, msg: SetCapabilityUrls, _: &mut Self::Context) -> Self::Result {
        if let Some(session) = &mut self.session {
            session.capability_urls.extend(msg.capability_urls);
        }
    }
}

impl Handler<SendCapabilityRequest> for Mailbox {
    type Result = ();
    fn handle(&mut self, msg: SendCapabilityRequest, ctx: &mut Self::Context) -> Self::Result {
        if let Some(session) = &self.session {
            let seed_capability_url = session.seed_capability_url.clone();
            let address = ctx.address().clone();
            ctx.spawn(
                async move {
                    let client = awc::Client::default();
                    match client
                        .post(seed_capability_url)
                        .insert_header(("Content-Type", "application/llsd+xml"))
                        .send_body(msg.capability_request.capabilities)
                        .await
                    {
                        Ok(mut get) => match get.body().await {
                            Ok(body) => {
                                match CapabilityRequest::response_from_llsd(&body) {
                                    Ok(capability_urls) => {
                                        address.do_send(SetCapabilityUrls { capability_urls })
                                    }
                                    Err(e) => {
                                        error!("Capabilities failed to parse: {:?}: {:?}", e, body)
                                    }
                                };
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
        }
    }
}

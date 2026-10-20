use super::session::Mailbox;
use crate::session::SendUIMessage;
use actix::WrapFuture;
use actix::{AsyncContext, Handler, Message};
use benthic_protocol::messages::world::land_update::LandUpdate;
use benthic_protocol::messages::{ui_protocol::UIMessage, world::skybox_update::SkyboxUpdate};
use log::error;
use log::info;
use metaverse_environment::layer_handler::handle_layer;
use metaverse_environment::sim_time::fetch_environment_time;
use metaverse_messages::http::capabilities::Capability;
use metaverse_messages::udp::environment::layer_data::LayerData;

/// Handles received SimulatorVIewerTimeMessages
///
/// This message contains information about where the sun is in the sky
///
/// # Cause
/// - SimulatorViewerTimeMessage packet received from UDP socket on server
///
/// # Effects
/// - Dispatches a [`SkyboxUpdate`] message to inform the UI of sun movement
#[derive(Message)]
#[rtype(result = "()")]
pub struct HandleSimulatorViewerTimeMessage {
    /// Seconds since start of sim
    pub seconds_since_start: u64,
    /// Phase the sun is currently in
    pub sun_phase: f32,
    /// Time it takes for the sun to make one full revolution
    pub seconds_per_day: u32,
    /// Seconds per year to handle custom season change
    pub seconds_per_year: u32,
}
#[cfg(feature = "environment")]
impl Handler<HandleSimulatorViewerTimeMessage> for Mailbox {
    type Result = ();

    fn handle(
        &mut self,
        msg: HandleSimulatorViewerTimeMessage,
        ctx: &mut Self::Context,
    ) -> Self::Result {
        if let Some(session) = &mut self.session {
            let region = &mut session.region_data;

            if msg.seconds_since_start <= region.last_time_update {
                info!("Received out of order SimulatorViewerTimeMessage");
                return;
            }

            ctx.address().do_send(SendUIMessage {
                ui_message: UIMessage::new_skybox_update(SkyboxUpdate {
                    sun_phase: msg.sun_phase,
                }),
            });
        }
    }
}

/// Message to trigger the fetching of the day cycle data from the endpoint
///
/// # Cause
/// - `session.rs` 's handle login. Sent immediately after a successful login.
#[derive(Message)]
#[rtype(result = "()")]
pub struct FetchEnvironmentEvent {}
impl Handler<FetchEnvironmentEvent> for Mailbox {
    type Result = ();
    fn handle(&mut self, msg: FetchEnvironmentEvent, ctx: &mut Self::Context) -> Self::Result {
        let server_endpoint = self.session.as_mut().and_then(|session| {
            session
                .capability_urls
                .get(&Capability::ExtEnvironment)
                .cloned()
        });
        let Some(server_endpoint) = server_endpoint else {
            self.queue_for_capability(Capability::ExtEnvironment, msg);
            return;
        };
        let Some(_session) = self.session.as_ref() else {
            return;
        };

        ctx.spawn(
            async move {
                match fetch_environment_time(&server_endpoint).await {
                    Ok(_) => {}
                    Err(e) => error!("{:?}", e),
                }
            }
            .into_actor(self),
        );
    }
}

/// Message to handle new layer data coming in from the server
///
/// Handles land, water, cloud and wind patches. This decoding and handling is done in the
/// metaverse-environment crate. This is mainly used for maintaining the session's knowledge of land
/// tiles.  
///
/// # Cause
/// - LayerData packet received from UDP socket
///
/// # Effect
/// - Dispatches a [`LandUpdate`] packet to the UI
/// TODO: should dispatch more than a land update
#[derive(Message)]
#[rtype(result = "()")]
pub struct HandleLayerData {
    /// The layer data packet to process
    pub layer_data: LayerData,
}
impl Handler<HandleLayerData> for Mailbox {
    type Result = ();
    fn handle(&mut self, msg: HandleLayerData, ctx: &mut Self::Context) -> Self::Result {
        let Some(session) = self.session.as_mut() else {
            return;
        };

        match handle_layer(msg.layer_data, session) {
            Ok(paths) => {
                for path in paths {
                    ctx.address().do_send(SendUIMessage {
                        ui_message: UIMessage::new_land_update(LandUpdate { path }),
                    });
                }
            }
            Err(e) => {
                error!("{:?}", e);
            }
        };
    }
}

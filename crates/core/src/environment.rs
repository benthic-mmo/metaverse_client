#[cfg(feature = "environment")]
use crate::actors::SessionActor;
#[cfg(feature = "environment")]
use crate::ui_interactions::SendUIMessage;
use actix::WrapFuture;
use actix::{AsyncContext, Handler, Message};
use benthic_protocol::messages::world::land_update::LandUpdate;
use benthic_protocol::messages::{ui_protocol::UIMessage, world::skybox_update::SkyboxUpdate};
use log::{error, info};
use metaverse_environment::layer_handler::handle_layer;
use metaverse_environment::sim_time::fetch_environment_time;
use metaverse_messages::http::capabilities::Capability;
use metaverse_messages::udp::environment::layer_data::LayerData;

/// Handles received SimulatorViewerTimeMessages.
#[derive(Message)]
#[rtype(result = "()")]
pub struct HandleSimulatorViewerTimeMessage {
    /// Seconds since start of sim.
    pub seconds_since_start: u64,
    /// Phase the sun is currently in.
    pub sun_phase: f32,
    /// Time it takes for the sun to make one full revolution.
    pub seconds_per_day: u32,
    /// Seconds per year to handle custom season change.
    pub seconds_per_year: u32,
}

#[cfg(feature = "environment")]
impl Handler<HandleSimulatorViewerTimeMessage> for SessionActor {
    type Result = ();

    fn handle(
        &mut self,
        msg: HandleSimulatorViewerTimeMessage,
        _ctx: &mut Self::Context,
    ) -> Self::Result {
        let (Some(session), Some(interaction_actor)) =
            (self.session.as_mut(), self.interaction_actor.clone())
        else {
            return;
        };

        let region = &mut session.region_data;

        if msg.seconds_since_start <= region.last_time_update {
            info!("Received out of order SimulatorViewerTimeMessage");
            return;
        }

        interaction_actor.do_send(SendUIMessage {
            ui_message: UIMessage::new_skybox_update(SkyboxUpdate {
                sun_phase: msg.sun_phase,
            }),
        });
    }
}

/// Triggers fetching the day cycle data from the endpoint.
#[derive(Message)]
#[rtype(result = "()")]
pub struct FetchEnvironmentEvent {}

impl Handler<FetchEnvironmentEvent> for SessionActor {
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

        if self.session.is_none() {
            return;
        }

        let client = self.http_client.clone();

        ctx.spawn(
            async move {
                if let Err(e) = fetch_environment_time(&server_endpoint, client).await {
                    error!("{:?}", e);
                }
            }
            .into_actor(self),
        );
    }
}

/// Handles new layer data received from the server.
#[derive(Message)]
#[rtype(result = "()")]
pub struct HandleLayerData {
    /// The layer data packet to process.
    pub layer_data: LayerData,
}

impl Handler<HandleLayerData> for SessionActor {
    type Result = ();

    fn handle(&mut self, msg: HandleLayerData, _ctx: &mut Self::Context) -> Self::Result {
        let (Some(session), Some(interaction_actor)) =
            (self.session.as_mut(), self.interaction_actor.clone())
        else {
            return;
        };

        match handle_layer(msg.layer_data, session, self.session_context.dirs.clone()) {
            Ok(paths) => {
                for path in paths {
                    interaction_actor.do_send(SendUIMessage {
                        ui_message: UIMessage::new_land_update(LandUpdate { path }),
                    });
                }
            }
            Err(e) => {
                error!("{:?}", e);
            }
        }
    }
}

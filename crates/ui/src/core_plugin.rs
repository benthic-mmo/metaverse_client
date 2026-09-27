use benthic_protocol::messages::ui::camera_position::CameraPosition;
use benthic_protocol::messages::ui::chat_from_simulator::ChatFromSimulator;
use benthic_protocol::messages::ui::coarse_location_update::CoarseLocationUpdate;
use benthic_protocol::messages::ui::errors::SessionError;
use benthic_protocol::messages::ui::land_update::LandUpdate;
use benthic_protocol::messages::ui::login_error::LoginError;
use benthic_protocol::messages::ui::login_response::LoginResponse;
use benthic_protocol::messages::ui::mesh_update::MeshUpdate;
use benthic_protocol::messages::ui::play_animation::PlayAnimation;
use benthic_protocol::messages::ui::populate_inventory::PopulateInventory;
use benthic_protocol::messages::ui::skybox_update::SkyboxUpdate;
use benthic_protocol::messages::ui::ui_messages::UIMessage;
use benthic_protocol::messages::ui::water_update::WaterUpdate;
use bevy::app::App;
use bevy::prelude::*;

use crate::plugin::EventChannel;

#[derive(Message)]
pub struct LoginResponseEvent {
    pub value: Result<LoginResponse, LoginError>,
}

#[derive(Message)]
pub struct CameraUpdateEvent {
    pub value: CameraPosition,
}

#[derive(Message, Clone)]
pub struct PlayAnimationEvent {
    pub value: PlayAnimation,
}

#[derive(Message)]
pub struct CoarseLocationUpdateEvent {
    pub _value: CoarseLocationUpdate,
}

#[derive(Message)]
pub struct WaterUpdateEvent {
    pub value: WaterUpdate,
}

#[derive(Message)]
pub struct MeshUpdateEvent {
    pub value: MeshUpdate,
}

#[derive(Message)]
pub struct SkyboxUpdateEvent {
    pub value: SkyboxUpdate,
}

#[derive(Message)]
pub struct ChatMessageEvent {
    pub value: ChatFromSimulator,
}

#[derive(Message)]
pub struct AnimationEvent {
    pub value: PlayAnimation,
}

#[derive(Message)]
pub struct LandUpdateEvent {
    pub value: LandUpdate,
}

#[derive(Message)]
pub struct InventoryPopulateEvent {
    pub value: PopulateInventory,
}

#[derive(Message)]
pub struct DisableSimulatorEvent;

pub struct CorePlugin;
impl Plugin for CorePlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<LoginResponseEvent>()
            .add_message::<CameraUpdateEvent>()
            .add_message::<CoarseLocationUpdateEvent>()
            .add_message::<MeshUpdateEvent>()
            .add_message::<DisableSimulatorEvent>()
            .add_message::<SkyboxUpdateEvent>()
            .add_message::<WaterUpdateEvent>()
            .add_message::<LandUpdateEvent>()
            .add_message::<ChatMessageEvent>()
            .add_message::<AnimationEvent>()
            .add_message::<InventoryPopulateEvent>()
            .add_systems(Update, handle_queue);
    }
}

// Handle all of the core events that are received from the listener.
#[allow(clippy::all)]
fn handle_queue(
    event_channel: Res<EventChannel>,
    mut ev_loginresponse: MessageWriter<LoginResponseEvent>,
    mut ev_coarselocationupdate: MessageWriter<CoarseLocationUpdateEvent>,
    mut ev_disable_simulator: MessageWriter<DisableSimulatorEvent>,
    mut ev_mesh_update: MessageWriter<MeshUpdateEvent>,
    mut ev_land_update: MessageWriter<LandUpdateEvent>,
    mut ev_camera_update: MessageWriter<CameraUpdateEvent>,
    mut ev_water_update: MessageWriter<WaterUpdateEvent>,
    mut ev_skybox_update: MessageWriter<SkyboxUpdateEvent>,
    mut ev_chat_update: MessageWriter<ChatMessageEvent>,
    mut ev_animation_update: MessageWriter<AnimationEvent>,
    mut ev_inventory_update: MessageWriter<InventoryPopulateEvent>,
) {
    // Check for events in the channel
    let receiver = event_channel.receiver.clone();
    while let Ok(event) = receiver.try_recv() {
        match event {
            UIMessage::LandUpdate(land_update) => {
                ev_land_update.write(LandUpdateEvent { value: land_update });
            }
            UIMessage::LoginResponse(login_response) => {
                ev_loginresponse.write(LoginResponseEvent {
                    value: Ok(login_response),
                });
            }
            UIMessage::MeshUpdate(mesh_update) => {
                ev_mesh_update.write(MeshUpdateEvent { value: mesh_update });
            }
            UIMessage::PlayAnimation(play_animation) => {
                ev_animation_update.write(AnimationEvent {
                    value: play_animation,
                });
            }
            UIMessage::CoarseLocationUpdate(coarse_location_update) => {
                ev_coarselocationupdate.write(CoarseLocationUpdateEvent {
                    _value: coarse_location_update,
                });
            }
            UIMessage::ChatFromSimulator(chat_from_simulator) => {
                ev_chat_update.write(ChatMessageEvent {
                    value: chat_from_simulator,
                });
            }
            UIMessage::DisableSimulator(_) => {
                ev_disable_simulator.write(DisableSimulatorEvent {});
            }
            UIMessage::CameraPosition(data) => {
                ev_camera_update.write(CameraUpdateEvent { value: data });
            }
            UIMessage::WaterUpdate(data) => {
                ev_water_update.write(WaterUpdateEvent { value: data });
            }
            UIMessage::SkyboxUpdate(data) => {
                ev_skybox_update.write(SkyboxUpdateEvent { value: data });
            }
            UIMessage::PopulateInventory(data) => {
                ev_inventory_update.write(InventoryPopulateEvent { value: data });
            }
            UIMessage::Error(error) => match error {
                SessionError::Login(e) => {
                    ev_loginresponse.write(LoginResponseEvent {
                        value: Err(e.clone()),
                    });
                    error!("{:?}", e)
                }
                SessionError::MailboxSession(e) => {
                    info!("MailboxError {:?}", e)
                }
                SessionError::AckError(e) => {
                    info!("AckError {:?}", e)
                }
                SessionError::CircuitCode(e) => {
                    info!("CircuitcodeError {:?}", e)
                }
                SessionError::CompleteAgentMovement(e) => {
                    info!("CompleteAgentMovmentError {:?}", e)
                }
                SessionError::Capability(e) => {
                    info!("CapabilityError {:?}", e)
                }
                SessionError::IOError(e) => {
                    info!("IOError {:?}", e)
                }
                SessionError::FeatureError(e) => {
                    info!("FeatureError {:?}", e)
                }
            },
        };
    }
}

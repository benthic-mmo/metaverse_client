use bevy::gltf::{GltfMaterialName, GltfMeshName};
use bevy::mesh::skinning::SkinnedMesh;
use metaverse_core::initialize::initialize;
use metaverse_messages::packet::message::{UIMessage, UIResponse};
use std::fs::create_dir_all;
use std::net::UdpSocket;
use std::path::PathBuf;

use crate::errors::{NotLoggedIn, PacketSendError, PortError, ShareDirError};
use crate::render::{
    check_model_loaded, handle_land_update, handle_mesh_update, setup_environment, LandUpdateEvent,
    MeshQueue, MeshUpdateEvent,
};
use crate::subscriber::listen_for_core_events;
use crate::textures::environment::HeightMaterial;
use crate::{chat, login};
use actix_rt::System;
use bevy::app::App;
use bevy::prelude::*;
use bevy::tasks::AsyncComputeTaskPool;
use bevy::window::WindowCloseRequested;
use crossbeam_channel::{unbounded, Receiver, Sender};
use metaverse_messages::http::login::login_error::LoginError;
use metaverse_messages::udp::agent::coarse_location_update::CoarseLocationUpdate;
use metaverse_messages::ui::errors::SessionError;
use metaverse_messages::ui::login_response::LoginResponse;
use portpicker::pick_unused_port;

pub const VIEWER_NAME: &str = "benthic";

#[derive(Clone, Copy, Default, Eq, PartialEq, Debug, Hash, States)]
pub enum ViewerState {
    #[default]
    Login,
    Loading,
    Chat,
}

#[derive(Resource)]
pub struct Sockets {
    pub ui_to_core_socket: u16,
    pub core_to_ui_socket: u16,
}

#[derive(Resource)]
struct EventChannel {
    pub sender: Sender<UIMessage>,
    pub receiver: Receiver<UIMessage>,
}

#[derive(Resource)]
pub struct SessionData {
    login_response: Option<LoginResponse>,
}

#[derive(Resource)]
pub struct ChatMessages {
    pub messages: Vec<ChatFromClientMessage>,
}

#[derive(Resource)]
pub struct ShareDir {
    pub _path: PathBuf,
    pub login_cred_path: PathBuf,
}

pub struct ChatFromClientMessage {
    pub user: String,
    pub message: String,
}

#[derive(Message)]
pub struct LoginResponseEvent {
    pub value: Result<LoginResponse, LoginError>,
}

#[derive(Message)]
pub struct CoarseLocationUpdateEvent {
    pub _value: CoarseLocationUpdate,
}

#[derive(Resource)]
pub struct AgentUpdateTimer(Timer);

#[derive(Message)]
struct DisableSimulatorEvent;

#[derive(Message)]
struct LogoutRequestEvent;

//ensure the share dir exists, and create the Benthic folder in the share dir at startup time.
fn setup_share_dir() -> Result<PathBuf, ShareDirError> {
    let data_dir = dirs::data_dir().ok_or(ShareDirError::NoShareDir())?;
    let local_share_dir = data_dir.join(VIEWER_NAME);
    if !local_share_dir.exists() {
        create_dir_all(&local_share_dir)?
    };
    Ok(local_share_dir)
}

pub struct MetaversePlugin;
impl Plugin for MetaversePlugin {
    fn build(&self, app: &mut App) {
        let (s1, r1) = unbounded();
        let ui_to_core_socket =
            pick_unused_port().unwrap_or_else(|| panic!("{:?}", PortError::PortPickerError()));
        let core_to_ui_socket =
            pick_unused_port().unwrap_or_else(|| panic!("{:?}", PortError::PortPickerError()));

        let local_share_dir = match setup_share_dir() {
            Err(e) => {
                panic!("{:?}", e)
            }
            Ok(path) => {
                info!("Created directory: {:?}", path);
                path
            }
        };
        let login_cred_path = local_share_dir.join("login_conf.json");
        let login_data = match login::load_login_data(&login_cred_path) {
            Ok(login_data) => login_data,
            Err(e) => {
                error!("{:?}", e.to_string());
                login::LoginData::default()
            }
        };

        app.init_state::<ViewerState>()
            .add_plugins(MaterialPlugin::<HeightMaterial>::default())
            .insert_resource(SessionData {
                login_response: None,
            })
            .insert_resource(ChatMessages {
                messages: Vec::new(),
            })
            .insert_resource(Sockets {
                ui_to_core_socket,
                core_to_ui_socket,
            })
            .insert_resource(login_data)
            .insert_resource(chat::ChatMessage::default())
            .insert_resource(EventChannel {
                sender: s1,
                receiver: r1,
            })
            .insert_resource(ShareDir {
                _path: local_share_dir,
                login_cred_path,
            })
            .insert_resource(MeshQueue { items: vec![] })
            .add_message::<LoginResponseEvent>()
            .add_message::<CoarseLocationUpdateEvent>()
            .add_message::<MeshUpdateEvent>()
            .add_message::<LandUpdateEvent>()
            .add_message::<DisableSimulatorEvent>()
            .add_message::<LogoutRequestEvent>()
            .register_type::<Transform>()
            .register_type::<GlobalTransform>()
            .register_type::<TransformTreeChanged>()
            .register_type::<Children>()
            .register_type::<Visibility>()
            .register_type::<ChildOf>()
            .register_type::<InheritedVisibility>()
            .register_type::<ViewVisibility>()
            .register_type::<Name>()
            .register_type::<Mesh3d>()
            .register_type::<bevy::camera::primitives::Aabb>()
            .register_type::<SkinnedMesh>()
            .register_type::<GltfMeshName>()
            .register_type::<GltfMaterialName>()
            .add_systems(Startup, start_listener)
            .add_systems(Startup, setup_timers)
            .add_systems(Startup, setup_environment)
            .add_systems(Startup, start_core)
            .add_systems(Update, check_model_loaded)
            .add_systems(Update, handle_window_close)
            .add_systems(Update, handle_logout)
            .add_systems(Update, handle_queue)
            .add_systems(Update, handle_login_response)
            .add_systems(Update, handle_disconnect)
            .add_systems(Update, handle_mesh_update)
            .add_systems(Update, handle_land_update)
            .add_systems(
                Update,
                send_agent_update.run_if(in_state(ViewerState::Chat)),
            );
    }
}

pub fn handle_login_response(
    mut ev_loginresponse: MessageReader<LoginResponseEvent>,
    mut viewer_state: ResMut<NextState<ViewerState>>,
    mut session_data: ResMut<SessionData>,
) {
    for response in ev_loginresponse.read() {
        match response.value.as_ref() {
            Ok(login_response) => {
                viewer_state.set(ViewerState::Chat);
                session_data.login_response = Some(login_response.clone());
            }
            Err(_) => viewer_state.set(ViewerState::Login),
        }
    }
}

pub fn send_packet_to_core(packet: &[u8], sockets: &Res<Sockets>) -> Result<(), PacketSendError> {
    let client_socket = UdpSocket::bind("0.0.0.0:0")?;
    client_socket.send_to(packet, format!("127.0.0.1:{}", sockets.ui_to_core_socket))?;
    Ok(())
}

fn handle_window_close(
    mut events: MessageReader<WindowCloseRequested>,
    mut exit: MessageWriter<AppExit>,
    mut logout: MessageWriter<LogoutRequestEvent>,
    viewer_state: Res<State<ViewerState>>,
) {
    for _ in events.read() {
        info!("Window close requested. Exiting...");
        if *viewer_state == ViewerState::Chat {
            info!("Sending Logout");
            logout.write(LogoutRequestEvent);
        }
        exit.write(AppExit::Success);
    }
}
// required for AgentUpdate
fn setup_timers(mut commands: Commands) {
    commands.insert_resource(AgentUpdateTimer(Timer::from_seconds(
        0.1,
        TimerMode::Repeating,
    )));
}

fn handle_disconnect(
    mut ev_disable_simulator: MessageReader<DisableSimulatorEvent>,
    mut viewer_state: ResMut<NextState<ViewerState>>,
) {
    for _ in ev_disable_simulator.read() {
        viewer_state.set(ViewerState::Login);
    }
}

// Handle all of the core events that are received from the listener.
fn handle_queue(
    event_channel: Res<EventChannel>,
    mut ev_loginresponse: MessageWriter<LoginResponseEvent>,
    mut ev_coarselocationupdate: MessageWriter<CoarseLocationUpdateEvent>,
    mut ev_disable_simulator: MessageWriter<DisableSimulatorEvent>,
    mut ev_mesh_update: MessageWriter<MeshUpdateEvent>,
    mut ev_land_update: MessageWriter<LandUpdateEvent>,
    mut chat_messages: ResMut<ChatMessages>,
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
                info!("got LoginResponse")
            }
            UIMessage::MeshUpdate(mesh_update) => {
                ev_mesh_update.write(MeshUpdateEvent { value: mesh_update });
            }
            UIMessage::CoarseLocationUpdate(coarse_location_update) => {
                ev_coarselocationupdate.write(CoarseLocationUpdateEvent {
                    _value: coarse_location_update,
                });
                info!("got CoarseLocationUpdate")
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
            UIMessage::ChatFromSimulator(chat_from_simulator) => {
                chat_messages.messages.push(ChatFromClientMessage {
                    user: chat_from_simulator.from_name,
                    message: chat_from_simulator.message,
                });
            }
            UIMessage::DisableSimulator(_) => {
                ev_disable_simulator.write(DisableSimulatorEvent {});
            }
        };
    }
}

/// begin listening for UDP messages from the core
fn start_listener(sockets: Res<Sockets>, event_queue: Res<EventChannel>) {
    let outgoing_socket = sockets.core_to_ui_socket;
    let thread_pool = AsyncComputeTaskPool::get();
    let sender = event_queue.sender.clone();

    thread_pool
        .spawn(async move {
            listen_for_core_events(format!("127.0.0.1:{}", outgoing_socket), sender).await
        })
        .detach();
}

/// start the metaverse_core in a background thread
fn start_core(sockets: Res<Sockets>) {
    let server_to_ui_socket = sockets.core_to_ui_socket;
    let ui_to_server_socket = sockets.ui_to_core_socket;
    // start the actix process, and do not close the system until everything is finished
    std::thread::spawn(move || {
        System::new().block_on(async {
            match initialize(ui_to_server_socket, server_to_ui_socket).await {
                Ok(handle) => {
                    match handle.await {
                        Ok(()) => info!("Listener exited successfully!"),
                        Err(e) => error!("Listener exited with error {:?}", e),
                    };
                }
                Err(err) => {
                    error!("Failed to start client: {:?}", err);
                }
            }
        });
    });
}

fn handle_logout(mut events: MessageReader<LogoutRequestEvent>, sockets: Res<Sockets>) {
    for _ in events.read() {
        if let Err(e) = send_packet_to_core(&UIResponse::new_logout().to_bytes(), &sockets) {
            error!("{:?}", e)
        };
    }
}

pub fn retrieve_login_response<'a>(
    session_data: &'a Res<SessionData>,
    viewer_state: &mut ResMut<NextState<ViewerState>>,
) -> Result<&'a LoginResponse, NotLoggedIn> {
    match &session_data.login_response {
        Some(session) => Ok(session), // return owned copy
        None => {
            viewer_state.set(ViewerState::Login);
            error!("{:?}", NotLoggedIn::NotLoggedInError());
            Err(NotLoggedIn::NotLoggedInError())
        }
    }
}

// TODO: reimpliment this. This will send the location of the user to the server for motion, and
// needs to be handled by the ui.
fn send_agent_update(
    _session_data: Res<SessionData>,
    _sockets: Res<Sockets>,
    time: Res<Time>,
    mut timer: ResMut<AgentUpdateTimer>,
) {
    if !timer.0.tick(time.delta()).just_finished() {}

    //let data = session_data.login_response.as_ref().unwrap();
    //let packet = Packet::new_agent_update(AgentUpdate {
    //    agent_id: data.agent_id,
    //    session_id: data.session_id,
    //    ..Default::default()
    //})
    //.to_bytes();
    //println!("{:?}", packet);

    // let client_socket = UdpSocket::bind("0.0.0.0:0").unwrap();
    // match client_socket.send_to(
    //     &packet,
    //     format!("127.0.0.1:{}", sockets.ui_to_server_socket),
    // ) {
    //     Ok(_) => {}
    //     Err(e) => println!("Error sending agent update from UI {:?}", e),
    // };
}

use crate::animation::{BenthicAnimationPlugin, scene_instance_ready};
use crate::core_plugin::{CorePlugin, DisableSimulatorEvent, LoginResponseEvent};
use crate::environment::LandPlugin;
use crate::errors::{NotLoggedIn, PacketSendError, PortError, ShareDirError};
use crate::login;
use crate::mesh::BenthicMeshPlugin;
use crate::panels::chat_panel::ChatPlugin;
use crate::subscriber::listen_for_core_events;
use crate::water::WaterPlugin;
use actix_rt::System;
use benthic_protocol::messages::ui::agent_update::AgentUpdate;
use benthic_protocol::messages::ui::login_response::LoginResponse;
use benthic_protocol::messages::ui::ui_messages::{UIMessage, UIResponse};
use benthic_protocol::session::initialize_share_dir;
use bevy::app::App;
use bevy::prelude::*;
use bevy::tasks::AsyncComputeTaskPool;
use bevy::window::WindowCloseRequested;

use crossbeam_channel::{Receiver, Sender, unbounded};
use metaverse_core::initialize::initialize;
use portpicker::pick_unused_port;
use std::net::UdpSocket;
use std::path::PathBuf;

pub const VIEWER_NAME: &str = "benthic";

#[derive(Default, Resource, Clone)]
pub struct ChatMessage {
    pub message: String,
}

#[derive(Resource)]
pub struct AgentUpdateTimer(Timer);

#[derive(Resource)]
pub struct ShareDir {
    pub _path: PathBuf,
    pub login_cred_path: PathBuf,
}

#[derive(Resource)]
pub struct SessionData {
    pub login_response: Option<LoginResponse>,
    pub avatar_location: Vec3,
}

#[derive(Clone, Copy, Default, Eq, PartialEq, Debug, Hash, States)]
pub enum ViewerState {
    #[default]
    Login,
    Loading,
    Main,
}

#[derive(Resource)]
pub struct Sockets {
    pub ui_to_core_socket: u16,
    pub core_to_ui_socket: u16,
}

#[derive(Resource)]
pub struct EventChannel {
    pub sender: Sender<UIMessage>,
    pub receiver: Receiver<UIMessage>,
}

#[derive(Message)]
struct LogoutRequestEvent;

//ensure the share dir exists, and create the Benthic folder in the share dir at startup time.
fn setup_share_dir() -> Result<PathBuf, ShareDirError> {
    Ok(initialize_share_dir()?)
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
            .add_plugins(CorePlugin)
            .add_plugins(WaterPlugin)
            .add_plugins(ChatPlugin)
            //.add_plugins(SkyPlugin)
            .add_plugins(LandPlugin)
            .add_plugins(BenthicMeshPlugin)
            .add_plugins(BenthicAnimationPlugin)
            .insert_resource(login_data)
            .insert_resource(SessionData {
                login_response: None,
                avatar_location: Vec3::ZERO,
            })
            .insert_resource(Sockets {
                ui_to_core_socket,
                core_to_ui_socket,
            })
            .insert_resource(EventChannel {
                sender: s1,
                receiver: r1,
            })
            .insert_resource(ShareDir {
                _path: local_share_dir,
                login_cred_path,
            })
            .add_message::<LogoutRequestEvent>()
            .add_systems(Startup, start_listener)
            .add_systems(Startup, setup_timers)
            .add_systems(Startup, start_core)
            .add_systems(Update, handle_window_close)
            .add_systems(Update, handle_logout)
            .add_systems(Update, handle_login_response)
            .add_systems(Update, handle_disconnect)
            .add_systems(
                Update,
                send_agent_update.run_if(in_state(ViewerState::Main)),
            )
            .add_observer(scene_instance_ready);
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
                viewer_state.set(ViewerState::Main);
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
        if *viewer_state == ViewerState::Main {
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

// Send an agent update every second
fn send_agent_update(
    sockets: Res<Sockets>,
    time: Res<Time>,
    session: Res<SessionData>,
    mut timer: ResMut<AgentUpdateTimer>,
) {
    if !timer.0.tick(time.delta()).just_finished()
        && let Err(e) = send_packet_to_core(
            &UIResponse::new_agent_update(AgentUpdate {
                camera_center: session.avatar_location,
                ..Default::default()
            })
            .to_bytes(),
            &sockets,
        )
    {
        error!("{:?}", e)
    };
}

fn handle_logout(mut events: MessageReader<LogoutRequestEvent>, sockets: Res<Sockets>) {
    for _ in events.read() {
        if let Err(e) = send_packet_to_core(&UIResponse::new_logout().to_bytes(), &sockets) {
            error!("{:?}", e)
        };
    }
}

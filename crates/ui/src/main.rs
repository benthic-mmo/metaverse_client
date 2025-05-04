mod chat;
mod environment;
mod loading;
mod login;

use actix_rt::System;
use bevy::asset::UnapprovedPathMode;
use bevy_panorbit_camera::PanOrbitCameraPlugin;
use chat::chat_screen;
use crossbeam_channel::unbounded;
use crossbeam_channel::{Receiver, Sender};
use environment::{
    LayerUpdateEvent, PendingLayers, check_model_loaded, handle_layer_update, setup_environment,
};
use keyring::Entry;
use loading::loading_screen;
use login::login_screen;
use metaverse_messages::errors::SessionError;
use metaverse_messages::login_system::errors::LoginError;
use metaverse_messages::login_system::login_response::LoginResponse;
use metaverse_messages::packet_types::PacketType;
use metaverse_messages::ui::coarse_location_update::CoarseLocationUpdate;
use metaverse_session::client_subscriber::listen_for_server_events;
use portpicker::pick_unused_port;
use std::fs::{self, create_dir_all};
use std::path::PathBuf;

use bevy::{prelude::*, tasks::AsyncComputeTaskPool};
use bevy_egui::{EguiContexts, EguiPlugin, egui};
use metaverse_session::initialize::initialize;

#[derive(Resource)]
struct Sockets {
    ui_to_server_socket: u16,
    server_to_ui_socket: u16,
}

#[derive(Resource)]
struct EventChannel {
    sender: Sender<PacketType>,
    receiver: Receiver<PacketType>,
}

#[derive(Resource)]
struct SessionData {
    login_response: Option<LoginResponse>,
}

#[derive(Resource)]
struct ChatMessages {
    messages: Vec<ChatFromClientMessage>,
}

#[derive(Resource)]
struct ShareDir {
    path: Option<PathBuf>,
}

struct ChatFromClientMessage {
    user: String,
    message: String,
}

#[derive(Event)]
struct LoginResponseEvent {
    value: Result<LoginResponse, LoginError>,
}

#[derive(Event)]
struct CoarseLocationUpdateEvent {
    _value: CoarseLocationUpdate,
}

#[derive(Event)]
struct DisableSimulatorEvent;

#[derive(Clone, Copy, Default, Eq, PartialEq, Debug, Hash, States)]
enum ViewerState {
    #[default]
    Login,
    Loading,
    Chat,
}

pub const CONFIG_FILE: &str = "login_conf.json";
pub const VIEWER_NAME: &str = "BenthicViewer";

fn main() {
    let ui_to_server_socket = pick_unused_port().unwrap();
    let server_to_ui_socket = pick_unused_port().unwrap();
    let (s1, r1) = unbounded();
    let mut share_path = None;
    let mut login_data = login::LoginData::default();
    if let Some(data_dir) = dirs::data_dir() {
        let local_share_dir = data_dir.join("benthic");
        if !local_share_dir.exists() {
            if let Err(e) = create_dir_all(&local_share_dir) {
                warn!("Failed to create local share benthic : {}", e);
            };
            info!("Created Directory: {:?}", local_share_dir);
        }
        let file_path = local_share_dir.join(format!("login_conf.json",));
        share_path = Some(file_path.clone());
        login_data = match fs::read_to_string(file_path) {
            Ok(content) => match Entry::new(VIEWER_NAME, &content) {
                Ok(keyring) => match keyring.get_password() {
                    Ok(password) => {
                        let mut parts = content.split_whitespace();
                        let first_name = parts.next().unwrap_or_default().to_string();
                        let last_name = parts.next().unwrap_or_default().to_string();
                        let grid = parts.next().unwrap_or_default().to_string();
                        login::LoginData {
                            first_name,
                            last_name,
                            grid,
                            remember_me: true,
                            password,
                        }
                    }
                    Err(e) => {
                        println!("failed to get password from keyring: {:?}", e);
                        login::LoginData::default()
                    }
                },
                Err(e) => {
                    println!("failed to open keyring: {:?}", e);
                    login::LoginData::default()
                }
            },
            Err(e) => {
                println!("failed to read cached user data: {:?}", e);
                login::LoginData::default()
            }
        };
    }

    App::new()
        .add_plugins((DefaultPlugins.set(AssetPlugin {
            file_path: "assets".into(),
            unapproved_path_mode: UnapprovedPathMode::Allow,
            ..default()
        }),))
        .add_plugins(EguiPlugin {
            enable_multipass_for_primary_context: false,
        })
        .add_plugins(PanOrbitCameraPlugin)
        .add_systems(Startup, setup_environment)
        .add_systems(Startup, configure_visuals_system)
        // initial state of viewer is default, which is Login
        .init_state::<ViewerState>()
        .insert_resource(SessionData {
            login_response: None,
        })
        .insert_resource(ChatMessages {
            messages: Vec::new(),
        })
        .insert_resource(Sockets {
            ui_to_server_socket,
            server_to_ui_socket,
        })
        .insert_resource(ShareDir { path: share_path })
        .insert_resource(login_data)
        .insert_resource(chat::ChatMessage::default())
        .insert_resource(EventChannel {
            sender: s1,
            receiver: r1,
        })
        .insert_resource(PendingLayers { items: vec![] })
        //TODO: these should be in a plugin
        .add_systems(Startup, start_client)
        .add_systems(Startup, start_listener)
        .add_systems(Update, handle_queue)
        .add_systems(Update, handle_login_response)
        .add_systems(Update, handle_disconnect)
        .add_systems(Update, handle_layer_update)
        .add_event::<LoginResponseEvent>()
        .add_event::<CoarseLocationUpdateEvent>()
        .add_event::<LayerUpdateEvent>()
        .add_event::<DisableSimulatorEvent>()
        .add_systems(Update, login_screen.run_if(in_state(ViewerState::Login)))
        .add_systems(
            Update,
            loading_screen.run_if(in_state(ViewerState::Loading)),
        )
        .add_systems(Update, check_model_loaded)
        .add_systems(Update, chat_screen.run_if(in_state(ViewerState::Chat)))
        .run();
}

fn handle_login_response(
    mut ev_loginresponse: EventReader<LoginResponseEvent>,
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

fn handle_disconnect(
    mut ev_disable_simulator: EventReader<DisableSimulatorEvent>,
    mut viewer_state: ResMut<NextState<ViewerState>>,
) {
    for _ in ev_disable_simulator.read() {
        viewer_state.set(ViewerState::Login);
    }
}

fn configure_visuals_system(mut contexts: EguiContexts) {
    contexts.ctx_mut().set_visuals(egui::Visuals {
        ..Default::default()
    });
}

fn handle_queue(
    event_channel: Res<EventChannel>,
    mut ev_loginresponse: EventWriter<LoginResponseEvent>,
    mut ev_coarselocationupdate: EventWriter<CoarseLocationUpdateEvent>,
    mut ev_disable_simulator: EventWriter<DisableSimulatorEvent>,
    mut ev_layer_update: EventWriter<LayerUpdateEvent>,
    mut chat_messages: ResMut<ChatMessages>,
) {
    // Check for events in the channel
    let receiver = event_channel.receiver.clone();
    while let Ok(event) = receiver.try_recv() {
        match event {
            PacketType::LoginResponse(login_response) => {
                ev_loginresponse.write(LoginResponseEvent {
                    value: Ok(*login_response),
                });
                info!("got LoginResponse")
            }
            PacketType::LayerUpdate(layer_update) => {
                ev_layer_update.write(LayerUpdateEvent {
                    value: *layer_update,
                });
            }
            PacketType::CoarseLocationUpdate(coarse_location_update) => {
                ev_coarselocationupdate.write(CoarseLocationUpdateEvent {
                    _value: *coarse_location_update,
                });
                info!("got CoarseLocationUpdate")
            }
            PacketType::Error(error) => match *error {
                SessionError::Login(e) => {
                    ev_loginresponse.write(LoginResponseEvent { value: Err(e) });
                }
                SessionError::Mailbox(e) => {
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
            },
            PacketType::ChatFromSimulator(chat_from_simulator) => {
                chat_messages.messages.push(ChatFromClientMessage {
                    user: chat_from_simulator.from_name,
                    message: chat_from_simulator.message,
                });
            }
            PacketType::DisableSimulator(_) => {
                ev_disable_simulator.write(DisableSimulatorEvent {});
            }
            _ => {
                info!("unknown event coming from server")
            }
        };
    }
}

fn start_listener(sockets: Res<Sockets>, event_queue: Res<EventChannel>) {
    let outgoing_socket = sockets.server_to_ui_socket;
    let thread_pool = AsyncComputeTaskPool::get();
    let sender = event_queue.sender.clone();

    thread_pool
        .spawn(async move {
            listen_for_server_events(format!("127.0.0.1:{}", outgoing_socket), sender).await
        })
        .detach();
}

fn start_client(sockets: Res<Sockets>) {
    let server_to_ui_socket = sockets.server_to_ui_socket;
    let ui_to_server_socket = sockets.ui_to_server_socket;
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

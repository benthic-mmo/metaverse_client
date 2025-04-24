mod chat;
mod loading;
mod login;

use actix_rt::System;
use chat::chat_screen;
use crossbeam_channel::unbounded;
use crossbeam_channel::{Receiver, Sender};
use loading::loading_screen;
use login::login_screen;
use metaverse_messages::coarse_location_update::CoarseLocationUpdate;
use metaverse_messages::errors::SessionError;
use metaverse_messages::login_system::errors::LoginError;
use metaverse_messages::login_system::login_response::LoginResponse;
use metaverse_messages::packet_types::PacketType;
use metaverse_session::client_subscriber::listen_for_server_events;
use portpicker::pick_unused_port;

use bevy::{prelude::*, tasks::AsyncComputeTaskPool};
use bevy_egui::{egui, EguiContexts, EguiPlugin};
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

fn main() {
    // create temporary files

    let ui_to_server_socket = pick_unused_port().unwrap();
    let server_to_ui_socket = pick_unused_port().unwrap();
    let (s1, r1) = unbounded();

    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(EguiPlugin)
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
        .insert_resource(login::LoginData::default())
        .insert_resource(chat::ChatMessage::default())
        .insert_resource(EventChannel {
            sender: s1,
            receiver: r1,
        })
        //TODO: these should be in a plugin
        .add_systems(Startup, start_client)
        .add_systems(Startup, start_listener)
        .add_systems(Update, handle_queue)
        .add_systems(Update, handle_login_response)
        .add_systems(Update, handle_disconnect)
        .add_event::<LoginResponseEvent>()
        .add_event::<CoarseLocationUpdateEvent>()
        .add_event::<DisableSimulatorEvent>()
        .add_systems(Update, login_screen.run_if(in_state(ViewerState::Login)))
        .add_systems(
            Update,
            loading_screen.run_if(in_state(ViewerState::Loading)),
        )
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
        window_rounding: 0.0.into(),
        ..Default::default()
    });
}

fn handle_queue(
    event_channel: Res<EventChannel>,
    mut ev_loginresponse: EventWriter<LoginResponseEvent>,
    mut ev_coarselocationupdate: EventWriter<CoarseLocationUpdateEvent>,
    mut ev_disable_simulator: EventWriter<DisableSimulatorEvent>,
    mut chat_messages: ResMut<ChatMessages>,
) {
    // Check for events in the channel
    let receiver = event_channel.receiver.clone();
    while let Ok(event) = receiver.try_recv() {
        match event {
            PacketType::LoginResponse(login_response) => {
                ev_loginresponse.send(LoginResponseEvent {
                    value: Ok(*login_response),
                });
                info!("got LoginResponse")
            }
            PacketType::CoarseLocationUpdate(coarse_location_update) => {
                ev_coarselocationupdate.send(CoarseLocationUpdateEvent {
                    _value: *coarse_location_update,
                });
                info!("got CoarseLocationUpdate")
            }
            PacketType::Error(error) => match *error {
                SessionError::Login(e) => {
                    ev_loginresponse.send(LoginResponseEvent { value: Err(e) });
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
                ev_disable_simulator.send(DisableSimulatorEvent {});
            }
            _ => {
                info!("unknown event coming from server")
            }
        };
    }
}

fn start_listener(sockets: Res<Sockets>, event_queue: Res<EventChannel>) {
    let outgoing_socket = sockets.server_to_ui_socket.clone();
    let thread_pool = AsyncComputeTaskPool::get();
    let sender = event_queue.sender.clone();

    thread_pool
        .spawn(async move {
            listen_for_server_events(format!("127.0.0.1:{}", outgoing_socket), sender).await
        })
        .detach();
}

fn start_client(sockets: Res<Sockets>) {
    let server_to_ui_socket = sockets.server_to_ui_socket.clone();
    let ui_to_server_socket = sockets.ui_to_server_socket.clone();
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

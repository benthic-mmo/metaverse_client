mod login;
use std::path::PathBuf;

use actix_rt::System;
use crossbeam_channel::unbounded;
use crossbeam_channel::{Receiver, Sender};
use metaverse_messages::login_system::login_response::LoginResponse;
use metaverse_session::client_subscriber::listen_for_server_events;
use tempfile::NamedTempFile;

use bevy::{prelude::*, tasks::AsyncComputeTaskPool};
use bevy_egui::{egui, EguiContexts, EguiPlugin};
use metaverse_session::initialize::initialize;

#[derive(Resource)]
struct Sockets {
    ui_to_server_socket: PathBuf,
    server_to_ui_socket: PathBuf,
}

#[derive(Resource)]
struct EventChannel {
    sender: Sender<LoginResponse>,
    receiver: Receiver<LoginResponse>,
}

#[derive(Event)]
struct LoginResponseEvent {
    _value: LoginResponse,
}

fn main() {
    // create temporary files
    let ui_to_server_socket = NamedTempFile::new()
        .expect("Failed to create temp file")
        .path()
        .to_path_buf();
    let server_to_ui_socket = NamedTempFile::new()
        .expect("Failed to create temp file")
        .path()
        .to_path_buf();

    let (s1, r1) = unbounded();

    App::new()
        .insert_resource(Sockets {
            ui_to_server_socket,
            server_to_ui_socket,
        })
        .add_plugins(DefaultPlugins)
        .add_plugins(EguiPlugin)
        .insert_resource(login::LoginData::default())
        .add_systems(Startup, configure_visuals_system)
        .add_systems(Update, login::ui_login_system)
        .insert_resource(EventChannel {
            sender: s1,
            receiver: r1,
        })
        .add_systems(Startup, start_client)
        .add_systems(Startup, start_listener)
        .add_systems(Update, handle_queue)
        .add_event::<LoginResponseEvent>()
        .run();
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
) {
    // Check for events in the channel
    let receiver = event_channel.receiver.clone();
    while let Ok(event) = receiver.try_recv() {
        info!("EVENT RECEIVED");
        ev_loginresponse.send(LoginResponseEvent { _value: event });
    }
}

fn start_listener(sockets: Res<Sockets>, event_queue: Res<EventChannel>) {
    let outgoing_socket = sockets.server_to_ui_socket.clone();
    let thread_pool = AsyncComputeTaskPool::get();
    let sender = event_queue.sender.clone();

    thread_pool
        .spawn(async move { listen_for_server_events(outgoing_socket, sender).await })
        .detach();
}

fn start_client(sockets: Res<Sockets>) {
    let server_to_ui_socket = sockets.server_to_ui_socket.clone();
    let ui_to_server_socket = sockets.ui_to_server_socket.clone();
    // start the actix process, and do not close the system until everything is finished
    std::thread::spawn(|| {
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

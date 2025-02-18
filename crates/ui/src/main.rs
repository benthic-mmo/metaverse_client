mod login;
use std::path::PathBuf;
use std::os::unix::net::UnixDatagram;

use actix_rt::System;
use bevy::utils::HashMap;
use crossbeam_channel::unbounded;
use crossbeam_channel::{Sender, Receiver};
use metaverse_login::models::login_response::LoginResponse;
use metaverse_session::mailbox::TriggerSend;
use tempfile::NamedTempFile;

use bevy::{prelude::*, tasks::AsyncComputeTaskPool};
use bevy_egui::{egui, EguiContexts, EguiPlugin};
use metaverse_session::initialize::initialize;

#[derive(Resource)]
struct Sockets {
    incoming_socket: PathBuf,
    outgoing_socket: PathBuf,
}

#[derive(Resource)]
struct EventChannel {
    sender: Sender<LoginResponseEvent>,
    receiver: Receiver<LoginResponseEvent>,
}

#[derive(Event)]
struct LoginResponseEvent {
    value: LoginResponse,
}


fn main() {
    // create temporary files
    let incoming_socket_path = NamedTempFile::new()
        .expect("Failed to create temp file")
        .path()
        .to_path_buf();
    let outgoing_socket_path = NamedTempFile::new()
        .expect("Failed to create temp file")
        .path()
        .to_path_buf();

    let (s1, r1) = unbounded();

    App::new()
        .insert_resource(Sockets {
            incoming_socket: incoming_socket_path,
            outgoing_socket: outgoing_socket_path,
        })
        .add_plugins(DefaultPlugins)
        .add_plugins(EguiPlugin)
        .insert_resource(login::LoginData::default())
        .add_systems(Startup, configure_visuals_system)
        .add_systems(Startup, login::configure_ui_state_system)
        .add_systems(Update, login::ui_login_system)
        .insert_resource(Events::<login::ClientEvent>::default())
        .insert_resource(EventChannel{
            sender: s1,
            receiver: r1
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
    mut ev_loginresponse: EventWriter<LoginResponseEvent>
) {
    // Check for events in the channel
    let receiver = event_channel.receiver.clone();
    while let Ok(event) = receiver.try_recv() {
        info!("EVENT RECEIVED"); 
        ev_loginresponse.send(event);
    }
}

fn start_listener(sockets: Res<Sockets>, event_queue: Res<EventChannel>) {
    let outgoing_socket = sockets.outgoing_socket.clone();
    let socket = UnixDatagram::bind(outgoing_socket.clone()).unwrap();
    let thread_pool = AsyncComputeTaskPool::get();
    let sender = event_queue.sender.clone();

    thread_pool
        .spawn(async move {
            let mut message_store: HashMap<u16, String> = HashMap::new();
            info!(
                "Test client listening to outgoing UDS on: {:?}",
                outgoing_socket
            );
            loop {
                let mut buf = [0u8; 1024];
                match socket.recv_from(&mut buf) {
                    Ok((n, _)) => {
                        if let Ok(received_chunk) = TriggerSend::from_bytes(&buf[..n]) {
                            message_store
                                .entry(received_chunk.sequence_number)
                                .or_insert_with(String::new)
                                .push_str(&received_chunk.message);
                            // Check if we have all chunks
                            if message_store.len() == received_chunk.total_packet_number as usize {
                                let mut full_message = String::new();
                                for i in 0..received_chunk.total_packet_number {
                                    if let Some(chunk) = message_store.remove(&i) {
                                        full_message.push_str(&chunk);
                                    } else {
                                        error!("Missing chunk {} for message reconstruction", i);
                                        return;
                                    }
                                }
                                // we can do better than parsing this string but it's good enough for now
                                // After receiving the full message, check the message type and deserialize if needed
                                if received_chunk.message_type == "LoginResponse" {
                                    match serde_json::from_str::<LoginResponse>(&full_message) {
                                        Ok(login_response) => {
                                            {
                                                match sender.send(LoginResponseEvent{value: login_response}) {
                                                    Ok(()) => info!("sent LoginResponse event to the login response handler"),
                                                    Err(e) => error!("failed to send to mspc {:?}", e)
                                                };
                                            };
                                        }
                                        Err(e) => {
                                            error!("Failed to deserialize LoginResponse: {:?}", e);
                                        }
                                    }
                                }
                            }
                        } else {
                            info!("failed to deserialize the chunk :(")
                        }
                    }
                    Err(e) => {
                        error!("outgoing Failed to read buffer {}", e)
                    }
                }
            }
        })
        .detach();
}

fn start_client(sockets: Res<Sockets>) {
    let incoming_socket = sockets.incoming_socket.clone();
    let outgoing_socket = sockets.outgoing_socket.clone();
    // start the actix process, and do not close the system until everything is finished
    std::thread::spawn(|| {
            System::new().block_on(async {
                match initialize(incoming_socket, outgoing_socket).await {
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

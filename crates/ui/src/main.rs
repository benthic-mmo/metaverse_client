mod login;
use std::os::unix::net::UnixDatagram;
use std::path::PathBuf;

use actix_rt::System;
use tempfile::NamedTempFile;

use bevy::{prelude::*, tasks::AsyncComputeTaskPool};
use bevy_egui::{egui, EguiContexts, EguiPlugin};
use metaverse_session::initialize::initialize;

#[derive(Resource)]
struct Sockets {
    incoming_socket: PathBuf,
    outgoing_socket: PathBuf,
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
        .add_systems(Startup, start_client)
        .add_systems(Startup, start_listener)
        .run();
}

fn configure_visuals_system(mut contexts: EguiContexts) {
    contexts.ctx_mut().set_visuals(egui::Visuals {
        window_rounding: 0.0.into(),
        ..Default::default()
    });
}

fn start_listener(sockets: Res<Sockets>) {
    let thread_pool = AsyncComputeTaskPool::get();
    let outgoing_socket = sockets.outgoing_socket.clone();
    thread_pool
        .spawn(async move {
            let socket = UnixDatagram::bind(outgoing_socket.clone()).unwrap();
            info!(
                "UI process listening to outgoing UDS on: {:?}",
                outgoing_socket
            );
            loop {
                let mut buf = [0u8; 1024];
                match socket.recv_from(&mut buf) {
                    Ok((n, _)) => {
                        info!("UI process receiving data {}", n);
                    }
                    Err(e) => {
                        error!("UI process failed to read buffer {}", e)
                    }
                }
            }
        })
        .detach();
}

fn start_client(sockets: Res<Sockets>) {
    let incoming_socket = sockets.incoming_socket.clone();
    let outgoing_socket = sockets.outgoing_socket.clone();
    let thread_pool = AsyncComputeTaskPool::get();
    // start the actix process, and do not close the system until everything is finished
    thread_pool
        .spawn(async move {
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
        })
        .detach();
}

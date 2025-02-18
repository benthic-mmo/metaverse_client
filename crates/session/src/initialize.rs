use actix::Actor;
use log::error;
use std::collections::HashMap;
use std::sync::Mutex;
use std::{path::PathBuf, sync::Arc};
use tokio::sync::Notify;
use tokio::task::JoinHandle;

use crate::errors::SessionError;
use crate::listen_uds::listen;
use crate::mailbox::ServerState;
use crate::mailbox::{Mailbox, OutgoingSocket};

pub async fn initialize(
    incoming_socket_path: PathBuf,
    outgoing_socket_path: PathBuf,
) -> Result<JoinHandle<()>, SessionError> {
    let notify = Arc::new(Notify::new());
    let state = Arc::new(Mutex::new(ServerState::Starting));

    let mailbox = Mailbox {
        client_socket: 41519,
        outgoing_socket: None,
        packet_sequence_number: Arc::new(Mutex::new(0u32)),

        ack_queue: Arc::new(Mutex::new(HashMap::new())),
        update_stream: Arc::new(Mutex::new(Vec::new())),

        state: state.clone(),
        notify: notify.clone(),
        session: None,
    }
    .start();
    // wait until the mailbox starts
    notify.notified().await;
    if *state.lock().unwrap() != ServerState::Running {
        error!("server failed to start, also this isn't a circuitcode error, implement the real error k thx byyyeee")
    };

    let outgoing_socket = OutgoingSocket {
        socket_path: outgoing_socket_path,
    };
    if let Err(_) = mailbox.send(outgoing_socket).await {
        error!("Failed to bind to outgoing UDS. Your program will not receive communication from the server.")
    };

    let handle = actix::spawn(async move {
        listen(incoming_socket_path, mailbox).await;
    });

    Ok(handle)
}

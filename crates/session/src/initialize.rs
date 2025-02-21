use actix::Actor;
use std::collections::HashMap;
use std::sync::Mutex;
use std::{path::PathBuf, sync::Arc};
use tokio::sync::Notify;
use tokio::task::JoinHandle;

use crate::errors::{MailboxError, SessionError};
use crate::mailbox::ServerState;
use crate::mailbox::{Mailbox, ServerToUiSocket};
use crate::server_subscriber::listen_for_ui_messages;

/// This starts the mailbox, and blocks forever.
/// This should be run in its own thread, so as not to block anything else.
/// Also be sure that this is running within an actix system, or else it will fail silently.
///```
/// use metaverse_session::initialize::initialize;
/// use log::{info, error};
/// use tempfile::NamedTempFile;
/// use actix_rt::System;
///
/// let ui_to_server_socket = NamedTempFile::new()
///     .expect("Failed to create temp file")
///     .path()
///     .to_path_buf();
/// let server_to_ui_socket = NamedTempFile::new()
///     .expect("Failed to create temp file")
///     .path()
///     .to_path_buf();
/// std::thread::spawn(|| {
///    System::new().block_on(async {
///        match initialize(ui_to_server_socket, server_to_ui_socket).await {
///            Ok(handle) => {
///                match handle.await {
///                    Ok(()) => info!("Listener exited successfully!"),
///                    Err(e) => error!("Listener exited with error {:?}", e),
///                };
///            }
///            Err(err) => {
///                error!("Failed to start client: {:?}", err);
///            }
///        }
///    });
///});
///```
pub async fn initialize(
    ui_to_server_socket: PathBuf,
    server_to_ui_socket: PathBuf,
) -> Result<JoinHandle<()>, SessionError> {
    let notify = Arc::new(Notify::new());
    let state = Arc::new(Mutex::new(ServerState::Starting));

    let mailbox = Mailbox {
        // setting the port to 0 lets the OS choose an open port.
        client_socket: 0,
        server_to_ui_socket: None,
        packet_sequence_number: Arc::new(Mutex::new(0u32)),

        ack_queue: Arc::new(Mutex::new(HashMap::new())),

        state: state.clone(),
        notify: notify.clone(),
        session: None,
    }
    .start();
    // wait until the mailbox starts
    notify.notified().await;
    if *state.lock().unwrap() != ServerState::Running {
        return Err(SessionError::Mailbox(MailboxError {
            message: ("Mailbox failed to enter state Running.".to_string()),
        }));
    };

    let server_to_ui_socket = ServerToUiSocket {
        socket_path: server_to_ui_socket,
    };
    if let Err(e) = mailbox.send(server_to_ui_socket).await {
        return Err(SessionError::Mailbox(MailboxError {
            message: (format!("Failed to bind to outgoing UDS. {:?}", e)),
        }));
    };

    let handle = actix::spawn(async move {
        listen_for_ui_messages(ui_to_server_socket, mailbox).await;
    });

    Ok(handle)
}

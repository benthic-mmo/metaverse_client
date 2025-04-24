use actix::Actor;
use actix_rt::time;
use metaverse_messages::errors::{MailboxError, SessionError};
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::Mutex;
use std::time::Duration;
use tokio::sync::Notify;
use tokio::task::JoinHandle;

use crate::mailbox::Mailbox;
use crate::mailbox::{PingInfo, ServerState};
use crate::server_subscriber::listen_for_ui_messages;
use portpicker::pick_unused_port;

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
    ui_to_server_socket: u16,
    server_to_ui_socket: u16,
) -> Result<JoinHandle<()>, SessionError> {
    let notify = Arc::new(Notify::new());
    let state = Arc::new(Mutex::new(ServerState::Starting));

    let mailbox = Mailbox {
        client_socket: pick_unused_port().unwrap(),
        server_to_ui_socket: format!("127.0.0.1:{}", server_to_ui_socket),
        packet_sequence_number: Arc::new(Mutex::new(0u32)),

        ack_queue: Arc::new(Mutex::new(HashMap::new())),

        state: state.clone(),
        notify: notify.clone(),
        session: None,
        sent_packet_count: 0,
        ping_info: PingInfo {
            ping_number: 0,
            ping_latency: Duration::new(0, 0),
            last_ping: time::Instant::now(),
        },
    }
    .start();
    // wait until the mailbox starts
    notify.notified().await;
    if *state.lock().unwrap() != ServerState::Running {
        return Err(SessionError::Mailbox(MailboxError {
            message: ("Mailbox failed to enter state Running.".to_string()),
        }));
    };

    let handle = actix::spawn(async move {
        listen_for_ui_messages(format!("127.0.0.1:{}", ui_to_server_socket), mailbox).await;
    });

    Ok(handle)
}

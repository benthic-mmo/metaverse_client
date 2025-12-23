use crate::session::Mailbox;
use actix::Actor;
use actix_rt::time;
use log::error;
use metaverse_inventory::initialize_sqlite::init_sqlite;
use metaverse_messages::ui::errors::FeatureError;
use metaverse_messages::ui::errors::MailboxSessionError;
use metaverse_messages::ui::errors::SessionError;
use std::collections::HashSet;
use std::fs::create_dir_all;
use std::io;
use std::path::Path;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::Mutex;
use std::time::Duration;
use tokio::sync::Notify;
use tokio::task::JoinHandle;

use portpicker::pick_unused_port;

use crate::session::PingInfo;
use crate::session::ServerState;
use crate::transport::ui_event_listener::listen_for_ui_messages;

/// This starts the mailbox, and blocks forever.
/// This should be run in its own thread, so as not to block anything else.
/// Also be sure that this is running within an actix system, or else it will fail silently.
pub async fn initialize(
    ui_to_server_socket: u16,
    server_to_ui_socket: u16,
) -> Result<JoinHandle<()>, SessionError> {
    let notify = Arc::new(Notify::new());
    let state = Arc::new(Mutex::new(ServerState::Starting));

    let share_dir = initialize_share_dir()?;
    let db_path = share_dir.join("inventory.db");
    let connection = init_sqlite(db_path.clone())
        .await
        .map_err(|e| FeatureError::Inventory(format!("Failed to initialize SQLite: {}", e)))?;

    let mailbox = Mailbox {
        client_socket: pick_unused_port().unwrap(),
        server_to_ui_socket: format!("127.0.0.1:{}", server_to_ui_socket),
        inventory_db_connection: connection,
        inventory_db_location: db_path,

        server_acks: HashSet::new(),
        viewer_acks: HashSet::new(),

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
        return Err(SessionError::MailboxSession(MailboxSessionError {
            message: ("Mailbox failed to enter state Running.".to_string()),
        }));
    };

    let handle = actix::spawn(async move {
        listen_for_ui_messages(format!("127.0.0.1:{}", ui_to_server_socket), mailbox).await;
    });

    Ok(handle)
}

/// Ensure a directory exists
fn create_sub_dir(base: &Path, name: &str) -> io::Result<PathBuf> {
    let dir = base.join(name);
    create_dir_all(&dir).map_err(|e| {
        error!("Failed to create directory {:?}: {}", dir, e);
        e
    })?;
    Ok(dir)
}

/// Initialize the viewer's cache in the share dir on disk
pub fn initialize_share_dir() -> io::Result<PathBuf> {
    let data_dir = dirs::data_dir()
        .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "Failed to find data directory"))?;

    let share_dir = data_dir.join("benthic");
    create_dir_all(&share_dir).map_err(|e| {
        error!("Failed to create benthic share directory: {}", e);
        e
    })?;

    Ok(share_dir)
}

/// Create a subdirectory in the benthic share dir
pub fn create_sub_share_dir(name: &str) -> io::Result<PathBuf> {
    let share_dir = initialize_share_dir()?;
    create_sub_dir(&share_dir, name)
}

/// Create a subdirectory for user agents
pub fn create_sub_agent_dir(name: &str) -> io::Result<PathBuf> {
    let agent_dir = create_sub_share_dir("agent")?;
    create_sub_dir(&agent_dir, name)
}

/// Create a subdirectory for global objects
pub fn create_sub_object_dir(name: &str) -> io::Result<PathBuf> {
    let object_dir = create_sub_share_dir("object")?;
    create_sub_dir(&object_dir, name)
}

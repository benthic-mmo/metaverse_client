use crate::core::session::Mailbox;
use actix::Actor;
use actix_rt::time;
use log::error;
use log::info;
use metaverse_messages::ui::errors::MailboxSessionError;
use metaverse_messages::ui::errors::SessionError;
use std::collections::HashMap;
use std::fs;
use std::fs::create_dir_all;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::Mutex;
use std::time::Duration;
use tokio::sync::Notify;
use tokio::task::JoinHandle;

use portpicker::pick_unused_port;

use crate::core::session::PingInfo;
use crate::core::session::ServerState;
use crate::core_subscriber::listen_for_ui_messages;

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

    initialize_share_dir()?;

    #[cfg(feature = "agent")]
    initialize_skeleton()?;

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
        return Err(SessionError::MailboxSession(MailboxSessionError {
            message: ("Mailbox failed to enter state Running.".to_string()),
        }));
    };

    let handle = actix::spawn(async move {
        listen_for_ui_messages(format!("127.0.0.1:{}", ui_to_server_socket), mailbox).await;
    });

    Ok(handle)
}

/// Create a subdirectory in the benthic share dir.
/// if the directory already exists, it simply returns the path to the dir.
/// this is for creating new subfolders like "land" and "inventory" for downloaded assets.
pub fn create_sub_share_dir(dir: &str) -> std::io::Result<PathBuf> {
    let local_share_dir = initialize_share_dir()?;
    let new_dir = local_share_dir.join(dir);
    if !new_dir.exists() {
        if let Err(e) = create_dir_all(&new_dir) {
            error!("Failed to create {} dir {:?}", dir, e);
            return Err(e);
        };
        info!("Created Directory: {:?}", new_dir);
    }
    Ok(new_dir)
}

/// Initialize the viewer's cache in the share dir on disk
pub fn initialize_share_dir() -> std::io::Result<PathBuf> {
    if let Some(data_dir) = dirs::data_dir() {
        let local_share_dir = data_dir.join("benthic");
        if !local_share_dir.exists() {
            if let Err(e) = create_dir_all(&local_share_dir) {
                error!("Failed to create benthic share directory");
                return Err(e);
            };
            info!("Created Directory: {:?}", local_share_dir);
            Ok(local_share_dir)
        } else {
            Ok(local_share_dir)
        }
    } else {
        Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "Failed to find share dir",
        ))
    }
}

/// Initialize the local skeleton that contains joint rotations.
/// This is stored in agent/src/benthic_default_model/skeleton.gltf
/// If the file does not need to be copied to the share dir, then it simply returns the gltf file
/// already in share.
pub fn initialize_skeleton() -> std::io::Result<PathBuf> {
    let agent_dir = create_sub_share_dir("agent")?;
    let mut base_skeleton_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    base_skeleton_path.pop();
    base_skeleton_path.push("agent");
    base_skeleton_path.push("src");
    base_skeleton_path.push("benthic_default_model");
    base_skeleton_path.push("skeleton.gltf");

    let mut base_skeleton_bin_path = base_skeleton_path.clone();
    base_skeleton_bin_path.pop();
    base_skeleton_bin_path.push("skeleton.bin");

    let mut dest_gltf = agent_dir.clone();
    dest_gltf.push("skeleton.gltf");
    let mut dest_bin = agent_dir.clone();
    dest_bin.push("skeleton.bin");

    if !dest_bin.exists() {
        match fs::copy(&base_skeleton_bin_path, &dest_bin) {
            Ok(_) => info!("Copied skeleton.bin to {:?}", dest_bin),
            Err(_) => {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    "Failed to find copy skeleton.bin to agent share dir",
                ));
            }
        }
    }

    if !dest_gltf.exists() {
        match fs::copy(&base_skeleton_path, &dest_gltf) {
            Ok(_) => Ok(dest_gltf),
            Err(_) => {
                Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    "Failed to find copy skeleton.gltf to agent share dir",
                ))
            }
        }
    } else {
        Ok(dest_gltf)
    }
}

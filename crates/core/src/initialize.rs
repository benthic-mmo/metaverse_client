use crate::session::Mailbox;
use crate::session::PingInfo;
use crate::transport::ui_event_listener::listen_for_ui_messages;
use actix::Actor;
use actix_rt::time;
use benthic_protocol::errors::ProtocolSessionError;
use benthic_protocol::messages::simulator::error_message::MailboxSessionError;
use benthic_protocol::session::rate_limiter::RateLimiter;
use benthic_protocol::session::types::AssetKey;
use benthic_protocol::session::types::DownloadState;
use benthic_protocol::session::types::ServerState;
use metaverse_store::initialize_sqlite::Cache;
use metaverse_store::initialize_sqlite::init_surreal;
use portpicker::pick_unused_port;
use std::collections::HashMap;
use std::collections::HashSet;
use std::sync::Arc;
use std::sync::Mutex;
use std::time::Duration;
use tokio::sync::Mutex as tokioMutex;
use tokio::sync::Notify;
use tokio::sync::watch::Sender;
use tokio::task::JoinHandle;

/// This starts the mailbox, and blocks forever.
/// This should be run in its own thread, so as not to block anything else.
/// Also be sure that this is running within an actix system, or else it will fail silently.
pub async fn initialize(
    ui_to_server_socket: u16,
    server_to_ui_socket: u16,
) -> Result<JoinHandle<()>, ProtocolSessionError> {
    let notify = Arc::new(Notify::new());
    let state = Arc::new(Mutex::new(ServerState::Starting));
    let cache_db = init_surreal()
        .await
        .map_err(|err| std::io::Error::other(err.to_string()))?;
    let cache = Cache::new(cache_db);

    let http_client = awc::Client::default();

    let mailbox = Mailbox {
        client_socket: pick_unused_port().unwrap(),
        server_to_ui_socket: format!("127.0.0.1:{}", server_to_ui_socket),

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
        rate_limiter: Arc::new(RateLimiter::new(500, 50)),
        downloads: Arc::new(tokioMutex::new(
            HashMap::<AssetKey, Sender<DownloadState>>::new(),
        )),
        pending_children: Arc::new(tokioMutex::new(HashMap::new())),
        pending_avatar_objects: Vec::new(),
        pending_capability_messages: HashMap::new(),
        pending_animations: HashMap::new(),
        cache,

        http_client,
        permanent_failures: HashSet::new(),
        pending_avatars: HashSet::new(),
        flush_in_progress: false,
    }
    .start();
    notify.notified().await;
    if *state.lock().unwrap() != ServerState::Running {
        return Err(ProtocolSessionError::MailboxSession(MailboxSessionError {
            message: "Mailbox failed to enter state Running.".to_string(),
        }));
    }

    let handle = actix::spawn(async move {
        listen_for_ui_messages(format!("127.0.0.1:{}", ui_to_server_socket), mailbox).await;
    });

    Ok(handle)
}

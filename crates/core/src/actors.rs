use crate::avatar::AvatarObjectUpdate;
use crate::avatar::FinalizeAvatar;
use crate::avatar::HandleNewAvatarAnimation;
use crate::objects::RenderObjectMessage;
use crate::transport::ui_event_listener::listen_for_ui_messages;
use actix::Actor;
use actix::Addr;
use actix::Arbiter;
use actix::AsyncContext;
use actix::Context;
use actix::Handler;
use actix::Message;
use actix_rt::net::UdpSocket;
use awc::Client;
use benthic_protocol::errors::ProtocolSessionError;
use benthic_protocol::session::rate_limiter::RateLimiter;
use benthic_protocol::session::storage::initialize_share_dir;
use benthic_protocol::session::types::AssetKey;
use benthic_protocol::session::types::DownloadState;
use benthic_protocol::session::types::ServerState;
use benthic_protocol::session::types::Session;
use benthic_protocol::session::types::SessionContext;
use log::info;
use metaverse_avatar::avatar::Avatar;
use metaverse_environment::land::Land;
use metaverse_messages::http::capabilities::Capability;
use metaverse_store::initialize_sqlite::Cache;
use metaverse_store::initialize_sqlite::Inventory;
use metaverse_store::initialize_sqlite::init_surreal;
use portpicker::pick_unused_port;
use std::collections::HashMap;
use std::collections::HashSet;
use std::sync::Arc;
use std::sync::Mutex;
use std::time;
use std::time::Duration;
use tokio::sync::Mutex as tokioMutex;
use tokio::sync::Notify;
use tokio::sync::watch::{self};
use tokio::task::JoinHandle;
use uuid::Uuid;

type PendingSend = Box<dyn FnOnce(&Addr<SessionActor>) + Send>;
static AVATAR_RENDER_TIMEOUT: u64 = 10;
pub static DEFAULT_MAX_RETRIES: u32 = 50;

pub struct InteractionActor {
    state: Arc<Mutex<ServerState>>,
    notify: Arc<Notify>,

    pub mailbox_addr: Addr<SessionActor>,

    pub session_context: Arc<SessionContext>,
    /// The sequence number of the packets sent. Created as a simple count from the core to the server.
    pub sequence_number: u16,

    /// the global ping information
    pub ping_info: PingInfo,
    /// queue of packet IDs sent form the core to the server that the server hasn't yet acked
    pub viewer_acks: HashSet<u32>,
    /// queue of acks sent from the server to be responded to by the client
    pub server_acks: HashSet<u32>,

    /// the running UDP socket attached to the session  
    pub socket: Option<Arc<UdpSocket>>,

    /// the client socket for UDP connections
    pub client_socket: u16,

    /// UDP socket for connecting mailbox to the UI
    pub server_to_ui_socket: String,

    pub http_client: Client,
}
impl InteractionActor {
    pub fn new(
        mailbox_addr: Addr<SessionActor>,
        notify: Arc<Notify>,
        state: Arc<Mutex<ServerState>>,

        session_context: Arc<SessionContext>,
        server_to_ui_socket: u16,
    ) -> Self {
        InteractionActor {
            mailbox_addr,
            state,
            notify,
            session_context,
            sequence_number: 0,
            ping_info: PingInfo {
                ping_number: 0,
                ping_latency: Duration::new(0, 0),
                last_ping: time::Instant::now(),
            },
            server_acks: HashSet::new(),
            viewer_acks: HashSet::new(),
            socket: None,
            client_socket: pick_unused_port().unwrap(),
            server_to_ui_socket: format!("127.0.0.1:{}", server_to_ui_socket),
            http_client: awc::Client::default(),
        }
    }
    pub fn set_state(&mut self, new_state: ServerState, _ctx: &mut Context<Self>) {
        let state_clone: Arc<Mutex<ServerState>> = Arc::clone(&self.state);
        {
            let mut state = state_clone.lock().unwrap();
            *state = new_state.clone();
        }
        // notify on start and stop
        if new_state == ServerState::Running || new_state == ServerState::Stopped {
            self.notify.notify_one();
        }
    }
}
impl Actor for InteractionActor {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        info!("Interaction TID: {}", unsafe { libc::gettid() });
        info!(
            "Interaction Actor started on thread: {:?}",
            std::thread::current().id()
        );
        self.set_state(ServerState::Running, ctx);
    }
}
/// Information struct for storing latency and ping info
#[derive(Debug)]
pub struct PingInfo {
    /// the number of the ping
    pub ping_number: u8,
    /// how long the latency is. Currently not doing anything.
    pub ping_latency: Duration,
    /// time of last ping
    pub last_ping: time::Instant,
}

pub struct OptimizationActor {
    state: Arc<Mutex<ServerState>>,
    notify: Arc<Notify>,

    mailbox_addr: Addr<SessionActor>,
    session_context: Arc<SessionContext>,
}
impl OptimizationActor {
    pub fn new(
        mailbox_addr: Addr<SessionActor>,
        notify: Arc<Notify>,
        state: Arc<Mutex<ServerState>>,

        session_context: Arc<SessionContext>,
    ) -> Self {
        OptimizationActor {
            mailbox_addr,
            state,
            notify,
            session_context,
        }
    }
    pub fn set_state(&mut self, new_state: ServerState, _ctx: &mut Context<Self>) {
        let state_clone: Arc<Mutex<ServerState>> = Arc::clone(&self.state);
        {
            let mut state = state_clone.lock().unwrap();
            *state = new_state.clone();
        }
        // notify on start and stop
        if new_state == ServerState::Running || new_state == ServerState::Stopped {
            self.notify.notify_one();
        }
    }
}
impl Actor for OptimizationActor {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        info!("Optimization Actor TID: {}", unsafe { libc::gettid() });
        info!(
            "Optimization Actor started on thread: {:?}",
            std::thread::current().id()
        );
        self.set_state(ServerState::Running, ctx);
    }
}

/// Central Actix actor responsible for all client actix message handling within the session.
pub struct SessionActor {
    /// state of the mailbox. If it is running or not.
    pub state: Arc<Mutex<ServerState>>,
    /// notify for etablishing when it begins running
    pub notify: Arc<Notify>,
    /// Session information for after login
    pub session: Option<Session<Capability, Avatar, Land, Inventory>>,

    pub session_context: Arc<SessionContext>,
    /// the global number of packets that have been sent to the UI
    pub sent_packet_count: u16,

    /// Global rate limiter. This internally is a semaphore with a timer to prevent flooding the
    /// network with requests.
    pub rate_limiter: Arc<RateLimiter>,
    /// The list of currently downloading objects.
    /// this contains a UUID of the object downloading, along with a receiver that other
    /// objectupdats can subscribe to, in order to wake up when the download is complete.
    pub downloads: Arc<tokioMutex<HashMap<AssetKey, watch::Sender<DownloadState>>>>,
    /// RenderObjectMessages pending their parent to load in. The u32 is the local ID of the parent.
    pub pending_children: Arc<tokioMutex<HashMap<u32, Vec<RenderObjectMessage>>>>,
    /// Avatar objects pending because the current outfit is not loaded yet.
    pub pending_avatar_objects: Vec<AvatarObjectUpdate>,
    /// Messages that need capabilities to be present before sending
    pub pending_capability_messages: HashMap<Capability, Vec<PendingSend>>,
    /// animations that need an avatar to be rendered before sending
    pub pending_animations: HashMap<Uuid, Vec<HandleNewAvatarAnimation>>,

    /// Global awc client for requests. This allows connection pooling.
    pub http_client: Client,
    /// Storage for assets that should not be retried.
    /// this prevents sending new requests for assets that permanently failed on a different
    /// download attempt.
    pub permanent_failures: HashSet<AssetKey>,

    pub pending_avatars: HashSet<Uuid>,

    pub flush_in_progress: bool,

    pub interaction_actor: Option<Addr<InteractionActor>>,

    pub optimization_actor: Option<Addr<OptimizationActor>>,

    pub cache: Cache,
}
impl SessionActor {
    /// Set the state of the mailbox.
    /// Determines if it's running or started or stopped.
    pub fn set_state(&mut self, new_state: ServerState, _ctx: &mut Context<Self>) {
        let state_clone: Arc<Mutex<ServerState>> = Arc::clone(&self.state);
        {
            let mut state = state_clone.lock().unwrap();
            *state = new_state.clone();
        }
        // notify on start and stop
        if new_state == ServerState::Running || new_state == ServerState::Stopped {
            self.notify.notify_one();
        }
    }

    /// Add to the queue of messages waiting for capability endpoints.
    pub fn queue_for_capability<M>(&mut self, capability: Capability, msg: M)
    where
        M: Message<Result = ()> + Send + 'static,
        SessionActor: Handler<M>,
    {
        self.pending_capability_messages
            .entry(capability)
            .or_default()
            .push(Box::new(move |addr| {
                addr.do_send(msg);
            }));
    }

    /// Once the capabilities have loaded in, restart all of the pending messages
    pub fn flush_capability(&mut self, capability: Capability, ctx: &mut Context<Self>) {
        let Some(pending) = self.pending_capability_messages.remove(&capability) else {
            return;
        };

        let addr = ctx.address();

        for send in pending {
            send(&addr);
        }
    }

    /// Periodically check for avatars that have not received an object update in a while.
    /// when the timer runs out, render them.
    pub fn avatar_finished_loading(&self, ctx: &mut Context<Self>) {
        ctx.run_interval(std::time::Duration::from_millis(1000), |actor, ctx| {
            let finished: Vec<Uuid> = {
                let Some(session) = actor.session.as_ref() else {
                    return;
                };

                actor
                    .pending_avatars
                    .iter()
                    .filter(|id| {
                        session.avatars.get(id).is_some_and(|avatar| {
                            avatar.last_update.elapsed().is_ok_and(|elapsed| {
                                elapsed >= std::time::Duration::from_secs(AVATAR_RENDER_TIMEOUT)
                            })
                        })
                    })
                    .copied()
                    .collect()
            };

            for agent_id in finished {
                actor.pending_avatars.remove(&agent_id);
                ctx.address().do_send(FinalizeAvatar { agent_id });
            }
        });
    }
}
impl Actor for SessionActor {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        info!("Session Actor TID: {}", unsafe { libc::gettid() });
        info!(
            "Session Actor started on thread: {:?}",
            std::thread::current().id()
        );

        self.set_state(ServerState::Running, ctx);
        self.avatar_finished_loading(ctx);
    }
}
/// This starts the mailbox, and blocks forever.
/// This should be run in its own thread, so as not to block anything else.
/// Also be sure that this is running within an actix system, or else it will fail silently.
pub async fn initialize(
    ui_to_server_socket: u16,
    server_to_ui_socket: u16,
) -> Result<JoinHandle<()>, ProtocolSessionError> {
    let share_dir = initialize_share_dir()?;
    let session_context = Arc::new(SessionContext::new(share_dir));

    let cache_db = init_surreal()
        .await
        .map_err(|err| std::io::Error::other(err.to_string()))?;

    let mailbox_cache = Cache::new(cache_db);

    let mailbox_context = session_context.clone();

    let (mailbox, mailbox_arbiter) = start_actor("SessionActor", move |notify, state| {
        Ok(SessionActor {
            state,
            notify,
            session: None,
            session_context: mailbox_context,
            sent_packet_count: 0,

            rate_limiter: Arc::new(RateLimiter::new(500, 50)),
            downloads: Arc::new(tokioMutex::new(HashMap::new())),
            pending_children: Arc::new(tokioMutex::new(HashMap::new())),
            pending_avatar_objects: Vec::new(),
            pending_capability_messages: HashMap::new(),
            pending_animations: HashMap::new(),

            http_client: awc::Client::default(),
            permanent_failures: HashSet::new(),
            pending_avatars: HashSet::new(),
            flush_in_progress: false,

            interaction_actor: None,
            optimization_actor: None,

            cache: mailbox_cache,
        })
    })
    .await?;

    let interaction_context = session_context.clone();
    let mailbox_for_interaction = mailbox.clone();

    let (interaction_addr, interaction_arbiter) =
        start_actor("InteractionActor".into(), move |notify, state| {
            Ok(InteractionActor::new(
                mailbox_for_interaction,
                notify,
                state,
                interaction_context,
                server_to_ui_socket,
            ))
        })
        .await?;

    let optimizer_context = session_context.clone();
    let mailbox_for_optimizer = mailbox.clone();

    let (optimizer_addr, optimizer_arbiter) =
        start_actor("OptimizationActor".into(), move |notify, state| {
            Ok(OptimizationActor::new(
                mailbox_for_optimizer,
                notify,
                state,
                optimizer_context,
            ))
        })
        .await?;

    mailbox.do_send(AttachInteraction {
        addr: interaction_addr.clone(),
    });

    mailbox.do_send(AttachOptimizer {
        addr: optimizer_addr.clone(),
    });

    let handle = actix::spawn(async move {
        // Keep all arbiters alive for the duration of the session.
        let _arbiters = (mailbox_arbiter, interaction_arbiter, optimizer_arbiter);

        listen_for_ui_messages(
            format!("127.0.0.1:{}", ui_to_server_socket),
            interaction_addr,
        )
        .await;
    });

    Ok(handle)
}

async fn start_actor<A>(
    name: &'static str,
    ctor: impl FnOnce(Arc<Notify>, Arc<Mutex<ServerState>>) -> Result<A, ProtocolSessionError>
    + Send
    + 'static,
) -> Result<(Addr<A>, Arbiter), ProtocolSessionError>
where
    A: Actor<Context = Context<A>>,
{
    let notify = Arc::new(Notify::new());
    let state = Arc::new(Mutex::new(ServerState::Starting));

    let (tx, rx) = tokio::sync::oneshot::channel();
    let arbiter = Arbiter::new();

    let actor_notify = notify.clone();
    let actor_state = state.clone();

    arbiter.spawn(async move {
        let result = ctor(actor_notify, actor_state).map(Actor::start);

        let _ = tx.send(result);
    });

    let addr = rx
        .await
        .map_err(|_| ProtocolSessionError::StartupError(name.into()))??;

    notify.notified().await;

    if *state.lock().unwrap() != ServerState::Running {
        return Err(ProtocolSessionError::StartupError(name.into()));
    }

    Ok((addr, arbiter))
}

#[derive(Debug, Message)]
#[rtype(result = "()")]
pub struct AttachInteraction {
    pub addr: Addr<InteractionActor>,
}
impl Handler<AttachInteraction> for SessionActor {
    type Result = ();
    fn handle(&mut self, m: AttachInteraction, _: &mut Context<Self>) {
        self.interaction_actor = Some(m.addr);
    }
}

#[derive(Debug, Message)]
#[rtype(result = "()")]
pub struct AttachOptimizer {
    pub addr: Addr<OptimizationActor>,
}

impl Handler<AttachOptimizer> for SessionActor {
    type Result = ();
    fn handle(&mut self, m: AttachOptimizer, _: &mut Context<Self>) {
        self.optimization_actor = Some(m.addr);
    }
}

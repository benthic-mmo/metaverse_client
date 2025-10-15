use actix::prelude::*;
use actix_rt::time;
use log::{error, info};

use metaverse_agent::avatar::Avatar;
use metaverse_messages::http::capabilities::Capability;
use metaverse_messages::http::capabilities::CapabilityRequest;
use metaverse_messages::packet::message::UIMessage;
use metaverse_messages::packet::message::UIResponse;
use metaverse_messages::packet::packet::Packet;
use metaverse_messages::udp::chat::chat_from_viewer::ChatFromViewer;
use metaverse_messages::udp::core::circuit_code::CircuitCode;
use metaverse_messages::udp::core::complete_agent_movement::CompleteAgentMovementData;
use metaverse_messages::udp::core::complete_ping_check::CompletePingCheck;
use metaverse_messages::udp::core::logout_request::LogoutRequest;
use metaverse_messages::udp::core::packet_ack::PacketAck;
use metaverse_messages::udp::core::region_handshake_reply::RegionHandshakeReply;
use metaverse_messages::ui::errors::CapabilityError;
use metaverse_messages::ui::errors::CircuitCodeError;
use metaverse_messages::ui::errors::CompleteAgentMovementError;
use metaverse_messages::ui::errors::MailboxSessionError;
use metaverse_messages::ui::errors::SessionError;
use metaverse_messages::ui::login_event::Login;
use std::collections::HashMap;
use std::collections::HashSet;
use std::net::UdpSocket as SyncUdpSocket;
use std::sync::Arc;
use std::sync::Mutex;
use tokio::net::UdpSocket;
use tokio::sync::Notify;
use tokio::time::Duration;
use uuid::Uuid;

#[cfg(feature = "inventory")]
use crate::core::inventory::RefreshInventoryEvent;
use crate::http_handler::login_to_simulator;

use super::{environment::EnvironmentCache, inventory::InventoryData};

/// This is the mailbox for handling packets and sessions in the client
#[derive(Debug)]
pub struct Mailbox {
    /// the client socket for UDP connections
    pub client_socket: u16,
    /// UDP socket for connecting mailbox to the UI
    pub server_to_ui_socket: String,

    /// queue of ack packets to handle
    pub ack_queue: Arc<Mutex<HashSet<u32>>>,

    /// state of the mailbox. If it is running or not.
    pub state: Arc<Mutex<ServerState>>,
    /// notify for etablishing when it begins running
    pub notify: Arc<Notify>,
    /// Session information for after login
    pub session: Option<Session>,

    /// the global number of packets that have been sent to the UI
    pub sent_packet_count: u16,

    /// the global ping information
    pub ping_info: PingInfo,
}

/// Session of the user
#[derive(Debug, Message)]
#[rtype(result = "()")]
pub struct Session {
    /// address of the server the client is connected to. formatted http://Url:Socket
    pub address: String,
    /// agent ID of the user
    pub agent_id: Uuid,
    /// session ID of the user
    pub session_id: Uuid,
    /// the running UDP socket attached to the session  
    pub socket: Option<Arc<UdpSocket>>,
    /// The URL endpoint to request more capabilities
    pub seed_capability_url: String,
    /// The HashMap for storing capability URLs
    pub capability_urls: HashMap<Capability, String>,

    /// this stores information about the Inventory, like the rootID and the current inventory
    /// tree in memory.
    #[cfg(feature = "inventory")]
    pub inventory_data: InventoryData,

    /// The environment cache. Contains things for handling and generating the environment.
    #[cfg(feature = "environment")]
    pub environment_cache: EnvironmentCache,

    /// The agent list. Contains information about the appearances of all loaded agents
    #[cfg(feature = "agent")]
    pub agent_list: Arc<Mutex<HashMap<Uuid, Avatar>>>,
}

/// contains information about pings sent to the server
#[derive(Debug, Message)]
#[rtype(result = "()")]
pub struct PingInfo {
    /// the number of the ping
    pub ping_number: u8,
    /// how long the latency is. Currently not doing anything.
    pub ping_latency: Duration,
    /// time of last ping
    pub last_ping: time::Instant,
}

/// this is a simple message that gets sent when receiving the CompletePingcheck
#[derive(Debug, Message)]
#[rtype(result = "()")]
pub struct Ping {
    /// The ID of the ping
    pub ping_id: u8,
}

/// message to send when receiving a RegionHandshake
#[derive(Debug, Message)]
#[rtype(result = "()")]
pub struct RegionHandshakeMessage;

/// The state of the Mailbox
#[derive(Debug, Clone, PartialEq)]
pub enum ServerState {
    /// The mailbox starts in the Starting state
    Starting,
    /// The mailbox is running
    Running,
    /// The mailbox is preparing to stop
    Stopping,
    /// the mailbox is stopped
    Stopped,
}

/// Message used for acknowledging reliable headers for incoming packets.
/// When a UDP packet is received from the server with a reliable header, its sequence number is
/// sent back to the server in a PacketAck, so the server knows that sequence number was received.
/// Then the ack queue is cleared, to prevent sending acks for packets that have already been
/// received.
#[derive(Debug, Message)]
#[rtype(result = "()")]
pub struct SendAckList {}

impl Mailbox {
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
}

impl Actor for Mailbox {
    type Context = Context<Self>;
    fn started(&mut self, ctx: &mut Self::Context) {
        info!("Actix Mailbox has started");
        self.set_state(ServerState::Running, ctx);
    }
}

impl Handler<RegionHandshakeMessage> for Mailbox {
    type Result = ();
    fn handle(&mut self, _: RegionHandshakeMessage, ctx: &mut Self::Context) -> Self::Result {
        ctx.address()
            .do_send(Packet::new_region_handshake_reply(RegionHandshakeReply {
                session_id: self.session.as_ref().unwrap().session_id,
                agent_id: self.session.as_ref().unwrap().agent_id,
                flags: 0,
            }));
    }
}

impl Handler<Ping> for Mailbox {
    type Result = ();
    fn handle(&mut self, msg: Ping, ctx: &mut Self::Context) -> Self::Result {
        ctx.address()
            .do_send(Packet::new_complete_ping_check(CompletePingCheck {
                ping_id: msg.ping_id,
            }));
        self.ping_info.ping_latency = time::Instant::now() - self.ping_info.last_ping;
    }
}

impl Handler<UIMessage> for Mailbox {
    type Result = ();
    fn handle(&mut self, msg: UIMessage, _: &mut Self::Context) -> Self::Result {
        let client_socket = SyncUdpSocket::bind("0.0.0.0:0").unwrap();
        if let Err(e) = client_socket.send_to(&msg.to_bytes(), &self.server_to_ui_socket) {
            error!("Failed to send UI message:{:?}", e)
        }
    }
}

impl Handler<Session> for Mailbox {
    type Result = ();
    fn handle(&mut self, mut msg: Session, ctx: &mut Self::Context) -> Self::Result {
        if let Some(session) = self.session.as_ref() {
            msg.socket = session.socket.clone();
        }
        self.session = Some(msg);

        // if the session doesn't already have a UDP socket to watch, create one
        if let Some(session) = self.session.as_ref()
            && session.socket.is_none() {
                let addr = format!("0.0.0.0:{}", self.client_socket);

                let addr_clone = addr.clone();
                let mailbox_addr = ctx.address();

                info!("session established, starting UDP processing");
                let ack_queue = self.ack_queue.clone();

                let fut = async move {
                    match UdpSocket::bind(&addr).await {
                        Ok(sock) => {
                            info!("Successfully bound to {}", &addr);
                            let sock = Arc::new(sock);
                            // Spawn a new Tokio task for reading from the socket
                            tokio::spawn(Mailbox::start_udp_read(
                                ack_queue,
                                sock.clone(),
                                mailbox_addr,
                            ));
                            Ok(sock) // Return the socket wrapped in Arc
                        }
                        Err(e) => {
                            error!("Failed to bind to {}: {}", &addr_clone, e);
                            Err(e)
                        }
                    }
                };

                // wait for the socket to be successfully bound and then assign it
                ctx.spawn(fut.into_actor(self).map(|result, act, _| match result {
                    Ok(sock) => {
                        if let Some(session) = &mut act.session {
                            session.socket = Some(sock);
                        }
                    }
                    Err(_) => {
                        panic!("Socket binding failed");
                    }
                }));
            }
    }
}

impl Handler<UIResponse> for Mailbox {
    type Result = ();
    fn handle(&mut self, msg: UIResponse, ctx: &mut Self::Context) -> Self::Result {
        // handle login before session is established
        if let UIResponse::Login(data) = &msg {
            let ctx_addr = ctx.address().clone();
            let login_data = data.clone();
            actix::spawn(async move {
                if let Err(e) = handle_login(login_data, &ctx_addr).await {
                    error!("{:?}", e);
                };
            });
        }
        // other messages require session
        if let Some(ref session) = self.session {
            match msg {
                UIResponse::ChatFromViewer(data) => {
                    ctx.address()
                        .do_send(Packet::new_chat_from_viewer(ChatFromViewer {
                            session_id: session.session_id,
                            agent_id: session.agent_id,
                            channel: data.channel,
                            message: data.message,
                            message_type: data.message_type,
                        }));
                }
                UIResponse::Logout(_) => {
                    ctx.address()
                        .do_send(Packet::new_logout_request(LogoutRequest {
                            session_id: session.session_id,
                            agent_id: session.agent_id,
                        }));
                }
                data => {
                    error!("Unrecognized UIMessage: {:?}", data)
                }
            }
        }
    }
}

/// Handles sending packets to the server
impl Handler<Packet> for Mailbox {
    type Result = ();
    fn handle(&mut self, msg: Packet, ctx: &mut Self::Context) -> Self::Result {
        if let Some(ref session) = self.session {
            let addr = session.address.clone();
            let data = msg.to_bytes().clone();
            let socket_clone = session.socket.as_ref().unwrap().clone();
            let fut = async move {
                if let Err(e) = socket_clone.send_to(&data, &addr).await {
                    error!("Failed to send data: {}", e);
                }
            };
            ctx.spawn(fut.into_actor(self));
        }
    }
}

impl Handler<SendAckList> for Mailbox {
    type Result = ();
    fn handle(&mut self, _: SendAckList, ctx: &mut Self::Context) -> Self::Result {
        if let Some(ref session) = self.session {
            // send ack directly to the server
            let mut ack_queue = self.ack_queue.lock().unwrap();
            if ack_queue.is_empty() {
                return;
            }
            let packet_ids: Vec<u32> = ack_queue.drain().collect();

            let addr = session.address.clone();
            let packet = Packet::new_packet_ack(PacketAck { packet_ids }).to_bytes();
            let sock_clone = session.socket.clone().unwrap();
            let ack_wait = async move {
                if let Err(e) = sock_clone.send_to(&packet, addr).await {
                    println!("Failed to send ack: {:?}", e)
                };
            };
            ctx.spawn(ack_wait.into_actor(self));
        }
    }
}

async fn handle_login(
    login_data: Login,
    mailbox_addr: &actix::Addr<Mailbox>,
) -> Result<(), SessionError> {
    let login_response = match login_to_simulator(login_data).await {
        Ok(login_response) => {
            if let Err(e) = mailbox_addr
                .send(UIMessage::new_login_response_event(
                    metaverse_messages::ui::login_response::LoginResponse {
                        firstname: login_response.first_name.clone(),
                        lastname: login_response.last_name.clone(),
                    },
                ))
                .await
            {
                error!("Failed to send login response to UI {:?}", e)
            };
            login_response
        }
        Err(e) => {
            if let Err(e) = mailbox_addr
                .send(UIMessage::new_session_error(e.clone().into()))
                .await
            {
                error!("Failed to send session error: {:?}", e)
            };
            Err(e)?
        }
    };

    if let Err(e) = mailbox_addr
        .send(Session {
            agent_id: login_response.agent_id,
            session_id: login_response.session_id,
            address: format!("{}:{}", login_response.sim_ip, login_response.sim_port),
            seed_capability_url: login_response.seed_capability.unwrap(),
            capability_urls: HashMap::new(),

            #[cfg(feature = "environment")]
            environment_cache: EnvironmentCache {
                patch_queue: HashMap::new(),
                patch_cache: HashMap::new(),
            },

            #[cfg(feature = "inventory")]
            inventory_data: InventoryData {
                inventory_root: login_response.inventory_root,
                inventory_lib_root: login_response.inventory_lib_root,
                inventory_lib_owner: login_response.inventory_lib_owner,
                inventory_tree: None,
            },
            #[cfg(feature = "agent")]
            agent_list: Arc::new(Mutex::new(HashMap::new())),
            socket: None,
        })
        .await
    {
        Err(MailboxSessionError {
            message: e.to_string(),
        })?;
    };

    if let Err(e) = mailbox_addr
        .send(Packet::new_circuit_code(CircuitCode {
            code: login_response.circuit_code,
            session_id: login_response.session_id,
            id: login_response.agent_id,
        }))
        .await
    {
        Err(CircuitCodeError {
            message: e.to_string(),
        })?;
    };

    if let Err(e) = mailbox_addr
        .send(Packet::new_complete_agent_movement(
            CompleteAgentMovementData {
                circuit_code: login_response.circuit_code,
                session_id: login_response.session_id,
                agent_id: login_response.agent_id,
            },
        ))
        .await
    {
        Err(CompleteAgentMovementError {
            message: e.to_string(),
        })?;
    };
    match CapabilityRequest::new_capability_request(vec![
        #[cfg(any(feature = "agent", feature = "environment"))]
        Capability::ViewerAsset,
        #[cfg(feature = "inventory")]
        Capability::FetchLibDescendents2,
        #[cfg(feature = "inventory")]
        Capability::FetchInventoryDescendents2,
    ]) {
        Ok(caps) => {
            if let Err(e) = mailbox_addr.send(caps).await {
                Err(CapabilityError {
                    message: e.to_string(),
                })?
            }
        }
        Err(e) => {
            error!("{:?}", e)
        }
    };

    #[cfg(feature = "inventory")]
    if let Err(e) = mailbox_addr
        .send(RefreshInventoryEvent {
            agent_id: login_response.agent_id,
        })
        .await
    {
        Err(CapabilityError {
            message: e.to_string(),
        })?
    }
    Ok(())
}

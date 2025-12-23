use super::{environment::EnvironmentCache, inventory::InventoryData};
use crate::{
    capabilities::SendCapabilityRequest, inventory::RefreshInventoryEvent,
    transport::http_handler::login_to_simulator,
};
use actix::prelude::*;
use actix_rt::time;
use log::{error, info};
use metaverse_agent::avatar::Avatar;
use metaverse_messages::{
    http::capabilities::{Capability, CapabilityRequest},
    packet::{
        message::{UIMessage, UIResponse},
        packet::Packet,
    },
    udp::{
        chat::chat_from_viewer::ChatFromViewer,
        core::{
            agent_throttle::{AgentThrottle, ThrottleData},
            circuit_code::CircuitCode,
            complete_agent_movement::CompleteAgentMovementData,
            complete_ping_check::CompletePingCheck,
            logout_request::LogoutRequest,
            packet_ack::PacketAck,
            region_handshake::RegionHandshake,
            region_handshake_reply::RegionHandshakeReply,
        },
    },
    ui::{
        errors::{
            CapabilityError, CircuitCodeError, CompleteAgentMovementError, FeatureError,
            MailboxSessionError, SessionError,
        },
        login_event::Login,
    },
};
use sqlx::{Pool, Sqlite};
use std::{
    collections::{HashMap, HashSet},
    net::UdpSocket as SyncUdpSocket,
    path::PathBuf,
    sync::{Arc, Mutex},
    thread::sleep,
};
use tokio::{net::UdpSocket, sync::Notify, time::Duration};
use uuid::Uuid;

/// Central Actix actor responsible for all client actix message handling within the session.
#[derive(Debug)]
pub struct Mailbox {
    /// the client socket for UDP connections
    pub client_socket: u16,
    /// UDP socket for connecting mailbox to the UI
    pub server_to_ui_socket: String,
    /// the connection to the inventory sqlite DB
    /// this stores folder data and inventory metadata
    pub inventory_db_connection: Pool<Sqlite>,
    /// the location on disk of the inventory sqlite DB.
    pub inventory_db_location: PathBuf,
    /// queue of acks sent from the server to be responded to by the client
    pub server_acks: HashSet<u32>,
    /// queue of packet IDs sent form the core to the server that the server hasn't yet acked
    pub viewer_acks: HashSet<u32>,
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

/// Message and struct for the current user's session.
///
/// This includes all data that will be used throughout the session, and much of it is populated by
/// the LoginResponse packet. This sets the active session for the Mailbox, and ensures the UDP
/// socket doesn't close.
///
/// # Cause
/// - A login is triggered by the UI, and the handle_login function is called
///
/// # Effects
/// - Starts UDP read between client and server
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
    /// The sequence number of the packets sent. Created as a simple count from the core to the server.
    pub sequence_number: u16,
    /// The local IP that the login is sent from. This is stored to ensure the IP of the
    /// UseCircuitCode packet is sent from the same IP as the login, to prevent server errors.
    pub local_ip: std::net::IpAddr,
    /// The URL endpoint to request more capabilities
    pub seed_capability_url: String,
    /// The HashMap for storing capability URLs
    pub capability_urls: HashMap<Capability, String>,
    /// inventory details retrieved from initial login
    pub inventory_data: InventoryData,
    /// The environment cache. Contains things for handling and generating the environment.
    pub environment_cache: EnvironmentCache,
    /// The agent list. Contains information about the appearances of all loaded agents
    pub avatars: HashMap<Uuid, Avatar>,
}

/// Handles incoming pings from the server
///
/// # Cause
/// - Received StartPingCheck packet from UDP socket
#[derive(Debug, Message)]
#[rtype(result = "()")]
pub struct HandlePing {
    /// The ID of the ping
    pub ping_id: u8,
}

/// The state of the Mailbox, if it is running, starting, stopping or stopped.
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
///
/// When a UDP packet is received from the server with a reliable header, its sequence number is
/// sent back to the server in a PacketAck, so the server knows that sequence number was received.
/// Then the ack queue is cleared, to prevent sending acks for packets that have already been
/// received.
///
/// # Cause
/// - [`AddToAckList`]
///
/// # Effects
/// - Dispatches a [`PacketAck`] packet to the server
#[derive(Debug, Message)]
#[rtype(result = "()")]
pub struct SendAckList {}

/// Message for updating the list of unacked packets sent from the core to the server
///
/// Messages marked as reliable sent from the core must be resent until the server replies with a
/// PacketAck message. This adds thoes packets to the ack list.
///
/// # Cause
/// - Received a reliable packet from UDP socket
#[derive(Debug, Message)]
#[rtype(result = "()")]
pub struct AddToAckList {
    /// The ID of the packet to be acked
    pub id: u32,
}

/// Message for determining if a packet should be resent
///
/// When an outgoing packet is labeled reliable, this message is used to determine if it should be
/// resent to the server. An [`OutgoingPacket`] packet is sent initially, followed by a brief
/// timeout. This allows the server enough time to respond with an ack. If an ack is not received,
/// the packet is not resent. If it isn't, the packet will be resent.
///
/// # Cause
/// - [`OutgoingPacket`] on a reliable packet
///
/// # Effect
/// - [`OutgoingPacket`] if the ack was not received
#[derive(Debug, Message)]
#[rtype(result = "()")]
pub struct ResendPacket {
    /// the packet to resend
    pub packet: Packet,
}

/// Message for sending packets from the core to the server
///
/// Simply a wrapper for the packet struct to send UDP packets to the server
///
/// # Effect
/// - UDP packet sent to the server
/// - [`ResendPacket`] if the packet is marked reliable
#[derive(Debug, Message)]
#[rtype(result = "()")]
pub struct OutgoingPacket {
    /// The outgoing packet
    pub packet: Packet,
}

/// Message for handling region handshakes
///
/// Handles receiving region handshake data, and sending region handshake response packets.
///
/// # Cause
/// - Received a RegionHandshake packet from the UDP socket
///
/// # Effect
/// - Dispatches a [`RegionHandshakeReply`] packet to the server
#[derive(Debug, Message)]
#[rtype(result = "()")]
pub struct HandleRegionHandshake {
    /// The region hanshake data
    pub region_handshake: RegionHandshake,
}

/// Message for handling packet acks
///
/// Removes IDs from the outgoing packet ack queue. Received from the server to inform the core
/// that a reliable message sent from the core has successfully been received from the server
///
/// # Cause
/// - Received a PacketAck packet from the UDP socket
#[derive(Debug, Message)]
#[rtype(result = "()")]
pub struct HandlePacketAck {
    /// Ack data
    pub packet_ack: PacketAck,
}

/// Message for receiving updates from the UI to the core
///
/// # Cause
/// - Received a UIRespones message forom the UI-Core UDP socket
#[derive(Debug, Message)]
#[rtype(result = "()")]
pub struct HandleUIResponse {
    /// UI response data
    pub ui_response: UIResponse,
}

/// Message for sending data from the core to the UI
///
/// # Effect
/// - UDP packet sent to UI
#[derive(Debug, Message)]
#[rtype(result = "()")]
pub struct SendUIMessage {
    /// UI response data
    pub ui_message: UIMessage,
}

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

impl Handler<Session> for Mailbox {
    type Result = ();
    fn handle(&mut self, mut msg: Session, ctx: &mut Self::Context) -> Self::Result {
        if let Some(session) = self.session.as_ref() {
            msg.socket = session.socket.clone();
        }
        self.session = Some(msg);

        // if the session doesn't already have a UDP socket to watch, create one
        if let Some(session) = self.session.as_ref()
            && session.socket.is_none()
        {
            let addr = format!("{}:{}", session.local_ip, self.client_socket);

            let addr_clone = addr.clone();
            let mailbox_addr = ctx.address();

            info!("session established, starting UDP processing");

            let fut = async move {
                match UdpSocket::bind(&addr).await {
                    Ok(sock) => {
                        info!("Successfully bound to {}", &addr);
                        let sock = Arc::new(sock);
                        // Spawn a new Tokio task for reading from the socket
                        tokio::spawn(Mailbox::start_udp_read(
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

impl Handler<SendUIMessage> for Mailbox {
    type Result = ();
    fn handle(&mut self, msg: SendUIMessage, _: &mut Self::Context) -> Self::Result {
        let client_socket = SyncUdpSocket::bind("0.0.0.0:0").unwrap();
        if let Err(e) = client_socket.send_to(&msg.ui_message.to_bytes(), &self.server_to_ui_socket)
        {
            error!("Failed to send UI message:{:?}", e)
        }
    }
}

impl Handler<HandleUIResponse> for Mailbox {
    type Result = ();
    fn handle(&mut self, msg: HandleUIResponse, ctx: &mut Self::Context) -> Self::Result {
        // handle login before session is established
        if let UIResponse::Login(data) = &msg.ui_response {
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
            match msg.ui_response {
                UIResponse::ChatFromViewer(data) => {
                    ctx.address().do_send(OutgoingPacket {
                        packet: Packet::new_chat_from_viewer(ChatFromViewer {
                            session_id: session.session_id,
                            agent_id: session.agent_id,
                            channel: data.channel,
                            message: data.message,
                            message_type: data.message_type,
                        }),
                    });
                }
                UIResponse::Logout(_) => {
                    ctx.address().do_send(OutgoingPacket {
                        packet: Packet::new_logout_request(LogoutRequest {
                            session_id: session.session_id,
                            agent_id: session.agent_id,
                        }),
                    });
                }
                UIResponse::AgentUpdate(mut data) => {
                    data.agent_id = session.agent_id;
                    data.session_id = session.session_id;
                    ctx.address().do_send(OutgoingPacket {
                        packet: Packet::new_agent_update(data),
                    });
                }
                data => {
                    error!("Unrecognized UIMessage: {:?}", data)
                }
            }
        }
    }
}

/// Handles sending packets to the server
impl Handler<OutgoingPacket> for Mailbox {
    type Result = ();
    fn handle(&mut self, mut msg: OutgoingPacket, ctx: &mut Self::Context) -> Self::Result {
        if let Some(session) = self.session.as_mut() {
            let addr = session.address.clone();
            if !msg.packet.header.resent {
                msg.packet.header.sequence_number = session.sequence_number as u32;
                session.sequence_number += 1;
            }

            let data = msg.packet.to_bytes().clone();
            let socket_clone = session.socket.as_ref().unwrap().clone();
            let fut = async move {
                if let Err(e) = socket_clone.send_to(&data, &addr).await {
                    error!("Failed to send data: {}", e);
                }
            };
            ctx.spawn(fut.into_actor(self));

            // if the header is reliable, resend the packet until the viewer_acks contains the key
            if msg.packet.header.reliable {
                self.viewer_acks.insert(msg.packet.header.sequence_number);
                // give one second for the ack to come in.
                // the ResendPacket message check if viewer_acks still contains the sequence
                // number. if it doesn't, that means it's been removed by an ack. If it does,
                // that means it should be resent with the resent flag
                ctx.notify_later(ResendPacket { packet: msg.packet }, Duration::from_secs(1));
            };
        }
    }
}

impl Handler<ResendPacket> for Mailbox {
    type Result = ();
    fn handle(&mut self, mut msg: ResendPacket, ctx: &mut Self::Context) -> Self::Result {
        if self
            .viewer_acks
            .contains(&msg.packet.header.sequence_number)
        {
            msg.packet.header.resent = true;
            ctx.address().do_send(OutgoingPacket { packet: msg.packet });
        }
    }
}
impl Handler<HandleRegionHandshake> for Mailbox {
    type Result = ();
    fn handle(&mut self, _: HandleRegionHandshake, ctx: &mut Self::Context) -> Self::Result {
        ctx.address().do_send(OutgoingPacket {
            packet: Packet::new_region_handshake_reply(RegionHandshakeReply {
                session_id: self.session.as_ref().unwrap().session_id,
                agent_id: self.session.as_ref().unwrap().agent_id,
                flags: 0,
            }),
        });
    }
}

impl Handler<HandlePing> for Mailbox {
    type Result = ();
    fn handle(&mut self, msg: HandlePing, ctx: &mut Self::Context) -> Self::Result {
        ctx.address().do_send(OutgoingPacket {
            packet: Packet::new_complete_ping_check(CompletePingCheck {
                ping_id: msg.ping_id,
            }),
        });
        self.ping_info.ping_latency = time::Instant::now() - self.ping_info.last_ping;
    }
}

impl Handler<HandlePacketAck> for Mailbox {
    type Result = ();
    fn handle(&mut self, msg: HandlePacketAck, _ctx: &mut Self::Context) -> Self::Result {
        for id in msg.packet_ack.packet_ids {
            self.viewer_acks.remove(&id);
        }
    }
}

impl Handler<AddToAckList> for Mailbox {
    type Result = ();
    fn handle(&mut self, msg: AddToAckList, ctx: &mut Self::Context) -> Self::Result {
        self.server_acks.insert(msg.id);
        ctx.address().do_send(SendAckList {});
    }
}

impl Handler<SendAckList> for Mailbox {
    type Result = ();
    fn handle(&mut self, _: SendAckList, ctx: &mut Self::Context) -> Self::Result {
        if let Some(ref session) = self.session {
            // send ack directly to the server
            if self.server_acks.is_empty() {
                return;
            }
            let packet_ids: Vec<u32> = self.server_acks.drain().collect();

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
    let (login_response, local_ip) = match login_to_simulator(login_data).await {
        Ok((login_response, local_ip)) => {
            if let Err(e) = mailbox_addr
                .send(SendUIMessage {
                    ui_message: UIMessage::new_login_response_event(
                        metaverse_messages::ui::login_response::LoginResponse {
                            firstname: login_response.first_name.clone(),
                            lastname: login_response.last_name.clone(),
                        },
                    ),
                })
                .await
            {
                error!("Failed to send login response to UI {:?}", e)
            };
            (login_response, local_ip)
        }
        Err(e) => {
            if let Err(e) = mailbox_addr
                .send(SendUIMessage {
                    ui_message: UIMessage::new_session_error(e.clone().into()),
                })
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
            sequence_number: 0,
            local_ip,
            capability_urls: HashMap::new(),

            #[cfg(feature = "environment")]
            environment_cache: EnvironmentCache {
                patch_queue: HashMap::new(),
                patch_cache: HashMap::new(),
            },

            #[cfg(feature = "inventory")]
            inventory_data: InventoryData {
                inventory_root: login_response.inventory_root.ok_or_else(|| {
                    FeatureError::Inventory(
                        "Login response contained no inventory_root".to_string(),
                    )
                })?,
                inventory_lib_owner: login_response.inventory_lib_owner.ok_or_else(|| {
                    FeatureError::Inventory(
                        "Login response contained no inventory_lib_owner".to_string(),
                    )
                })?,
                inventory_init: false,
            },
            socket: None,

            #[cfg(feature = "agent")]
            avatars: HashMap::new(),
        })
        .await
    {
        Err(MailboxSessionError {
            message: e.to_string(),
        })?;
    };

    match CapabilityRequest::new_capability_request(vec![
        Capability::ViewerAsset,
        Capability::FetchInventoryDescendents2,
    ]) {
        Ok(caps) => {
            if let Err(e) = mailbox_addr
                .send(SendCapabilityRequest {
                    capability_request: caps,
                })
                .await
            {
                Err(CapabilityError {
                    message: e.to_string(),
                })?
            }
        }
        Err(e) => {
            error!("{:?}", e)
        }
    };

    if let Err(e) = mailbox_addr
        .send(OutgoingPacket {
            packet: Packet::new_circuit_code(CircuitCode {
                code: login_response.circuit_code,
                session_id: login_response.session_id,
                id: login_response.agent_id,
            }),
        })
        .await
    {
        Err(CircuitCodeError {
            message: e.to_string(),
        })?;
    };

    sleep(Duration::from_secs(1));
    if let Err(e) = mailbox_addr
        .send(OutgoingPacket {
            packet: Packet::new_complete_agent_movement(CompleteAgentMovementData {
                circuit_code: login_response.circuit_code,
                session_id: login_response.session_id,
                agent_id: login_response.agent_id,
            }),
        })
        .await
    {
        Err(CompleteAgentMovementError {
            message: e.to_string(),
        })?;
    };
    if let Err(e) = mailbox_addr
        .send(OutgoingPacket {
            packet: Packet::new_agent_throttle(AgentThrottle {
                agent_id: login_response.agent_id,
                session_id: login_response.session_id,
                circuit_code: login_response.circuit_code,
                gen_counter: 0,
                throttles: ThrottleData {
                    ..Default::default()
                },
            }),
        })
        .await
    {
        Err(CompleteAgentMovementError {
            message: e.to_string(),
        })?;
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

use actix::prelude::*;
use actix_rt::time;
use log::{error, info};

use metaverse_agent::avatar::Avatar;
use metaverse_messages::http::capabilities::Capability;
use metaverse_messages::packet::message::EventType;
use metaverse_messages::packet::message::UiMessage;
use metaverse_messages::packet::packet::Packet;
use metaverse_messages::udp::core::complete_ping_check::CompletePingCheck;
use metaverse_messages::udp::core::packet_ack::PacketAck;
use metaverse_messages::udp::core::region_handshake_reply::RegionHandshakeReply;
use std::collections::HashMap;
use std::collections::HashSet;
use std::net::UdpSocket as SyncUdpSocket;
use std::sync::Arc;
use std::sync::Mutex;
use tokio::net::UdpSocket;
use tokio::sync::Notify;
use tokio::time::Duration;
use uuid::Uuid;

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

impl Handler<UiMessage> for Mailbox {
    type Result = ();
    fn handle(&mut self, msg: UiMessage, _: &mut Self::Context) -> Self::Result {
        let max_message_size = 1024;
        // leave a little room at the end
        let overhead = 2;

        let sequence_number_len = std::mem::size_of::<u16>(); // 2 bytes for the sequence number
        let total_packet_number_len = std::mem::size_of::<u16>();
        let packet_number_len = std::mem::size_of::<u16>();

        // Calculate the maximum size available for the actual message content
        let available_size = max_message_size
            - (sequence_number_len + total_packet_number_len + packet_number_len + overhead);

        // Split the message content if it's larger than the available size
        let message = msg.message;
        let total_chunks = usize::max(1, message.len().div_ceil(available_size));

        // Loop through each chunk and send it
        for chunk_index in 0..total_chunks {
            let start = chunk_index * available_size;
            let end = usize::min(start + available_size, message.len());
            let chunk = &message[start..end];

            // Increment the sequence number for each chunk
            let sequence_number = msg.sequence_number + chunk_index as u16;

            // Create a new message with the chunked data
            let chunked_message = UiMessage {
                sequence_number,
                total_packet_number: total_chunks as u16, // Add total number of chunks
                message: chunk.to_vec(),
                packet_number: self.sent_packet_count,
            };

            let client_socket = SyncUdpSocket::bind("0.0.0.0:0").unwrap();
            if let Err(e) =
                client_socket.send_to(&chunked_message.to_bytes(), &self.server_to_ui_socket)
            {
                info!("sending to: {}", self.server_to_ui_socket);
                error!(
                    "Error sending chunk {} of {} from mailbox: {:?}",
                    sequence_number, total_chunks, e
                )
            }
        }
        self.sent_packet_count += 1;
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

impl Handler<EventType> for Mailbox {
    type Result = ();
    fn handle(&mut self, msg: EventType, ctx: &mut Self::Context) -> Self::Result {
        if let EventType::ChatFromViewer(data) = msg {
            ctx.address().do_send(Packet::new_chat_from_viewer(data));
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

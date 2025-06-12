use actix::prelude::*;
use actix_rt::time;
use bincode;
use log::{error, info};

use metaverse_agent::avatar::Avatar;
use metaverse_messages::capabilities::capabilities::Capability;
use metaverse_messages::core::complete_ping_check::CompletePingCheck;
use metaverse_messages::core::region_handshake_reply::RegionHandshakeReply;
use metaverse_messages::errors::errors::AckError;
use metaverse_messages::packet::packet::Packet;
use metaverse_messages::ui::errors::SessionError;
use metaverse_messages::ui::ui_events::UiEventTypes;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::net::UdpSocket as SyncUdpSocket;
use std::sync::Arc;
use std::sync::Mutex;
use tokio::net::UdpSocket;
use tokio::sync::{Notify, oneshot};
use tokio::time::Duration;
use uuid::Uuid;

use super::{environment::EnvironmentCache, inventory::InventoryData};

const ACK_ATTEMPTS: i8 = 3;
const ACK_TIMEOUT: Duration = Duration::from_secs(1);

/// This is the mailbox for handling packets and sessions in the client
#[derive(Debug)]
pub struct Mailbox {
    /// the client socket for UDP connections
    pub client_socket: u16,
    /// UDP socket for connecting mailbox to the UI
    pub server_to_ui_socket: String,

    /// queue of ack packets to handle
    pub ack_queue: Arc<Mutex<HashMap<u32, oneshot::Sender<()>>>>,

    /// global number of received packets
    pub packet_sequence_number: Arc<Mutex<u32>>,
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
    /// url of the server where the UDP session is connected to
    pub url: String,
    /// socket of the server where the UDP session is connected to
    pub server_socket: u16,
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

/// Format for sending a serialized message from the mailbox to the UI.
#[derive(Debug, Message, Serialize, Deserialize, Clone)]
#[rtype(result = "()")]
pub struct UiMessage {
    /// Type of message, for decoding in the UI
    pub message_type: UiEventTypes,
    /// Which number in a series of messages is it
    pub sequence_number: u16,
    /// how mant messages are there in total
    pub total_packet_number: u16,
    /// for serializing
    pub packet_number: u16,
    /// the encoded message to be decoded by the UI
    pub message: Vec<u8>,
}
impl UiMessage {
    /// Convert the struct into bytes using JSON serialization
    pub fn as_bytes(&self) -> Vec<u8> {
        bincode::serialize(self).expect("Failed to serialize UiMessage")
    }

    /// Convert bytes back into a `UiMessage` struct
    pub fn from_bytes(bytes: &[u8]) -> Option<Self> {
        bincode::deserialize(bytes).ok()
    }
    /// create a new UiMessage
    pub fn new(message_type: UiEventTypes, message: Vec<u8>) -> UiMessage {
        UiMessage {
            message_type,
            message,
            sequence_number: 0,
            total_packet_number: 0,
            packet_number: 0,
        }
    }
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

        let message_type_len = msg.message_type.to_string().len();
        let sequence_number_len = std::mem::size_of::<u16>(); // 2 bytes for the sequence number
        let total_packet_number_len = std::mem::size_of::<u16>();
        let packet_number_len = std::mem::size_of::<u16>();

        // Calculate the maximum size available for the actual message content
        let available_size = max_message_size
            - (message_type_len
                + sequence_number_len
                + total_packet_number_len
                + packet_number_len
                + overhead);

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
                message_type: msg.message_type.clone(),
                sequence_number,
                total_packet_number: total_chunks as u16, // Add total number of chunks
                message: chunk.to_vec(),
                packet_number: self.sent_packet_count,
            };

            let client_socket = SyncUdpSocket::bind("0.0.0.0:0").unwrap();
            if let Err(e) =
                client_socket.send_to(&chunked_message.as_bytes(), &self.server_to_ui_socket)
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
        if let Some(session) = self.session.as_ref() {
            if session.socket.is_none() {
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
}

/// This handles incoming packets sent to the Mailbox.
/// this sends acks if the header is reliable, and increases the sequence number.
impl Handler<Packet> for Mailbox {
    type Result = ();
    fn handle(&mut self, mut msg: Packet, ctx: &mut Self::Context) -> Self::Result {
        if let Some(ref session) = self.session {
            //TODO: This should really be saved somewhere
            let addr = format!("{}:{}", session.url, session.server_socket);

            {
                let sequence_number = self.packet_sequence_number.lock().unwrap();
                msg.header.sequence_number = *sequence_number;
            }

            if msg.header.reliable {
                let ack_future = send_ack(
                    msg,
                    addr,
                    self.ack_queue.clone(),
                    session.socket.as_ref().unwrap().clone(),
                );
                ctx.spawn(
                    async move {
                        if let Err(e) = ack_future.await {
                            error!("Error sending acknowledgment: {:?}", e);
                        }
                    }
                    .into_actor(self),
                );
            } else {
                let data = msg.to_bytes().clone();
                let socket_clone = session.socket.as_ref().unwrap().clone();
                let fut = async move {
                    if let Err(e) = socket_clone.send_to(&data, &addr).await {
                        error!("Failed to send data: {}", e);
                    }
                };
                ctx.spawn(fut.into_actor(self));
            };
            {
                let mut sequence_number = self.packet_sequence_number.lock().unwrap();
                *sequence_number += 1;
            }
        }
    }
}

async fn send_ack(
    packet: Packet,
    addr: String,
    ack_queue: Arc<Mutex<HashMap<u32, oneshot::Sender<()>>>>,
    socket: Arc<UdpSocket>,
) -> Result<(), SessionError> {
    let mut attempts = 0;
    let mut received_ack = false;
    let packet_id = packet.header.sequence_number;
    while attempts < ACK_ATTEMPTS && !received_ack {
        let (tx, rx) = oneshot::channel();
        let mut packet_clone = packet.clone();

        // if there have been more than 1 attempt, set the resent to true.
        if attempts > 0 {
            packet_clone.header.resent = true;
        }

        {
            let mut queue = ack_queue.lock().unwrap();
            queue.insert(packet_id, tx);
        }
        // Send the packet

        let data = packet_clone.to_bytes().clone();
        let addr_clone = addr.clone();
        let sock_clone = socket.clone();
        if let Err(e) = sock_clone.send_to(&data, addr_clone).await {
            error!("Failed to send Ack: {}", e);
        }

        tokio::select! {
            _ = rx => {
                received_ack = true;
            },
            _ = tokio::time::sleep(ACK_TIMEOUT) => {
                attempts += 1;
                if !received_ack && attempts >= ACK_ATTEMPTS {
                    // Remove from queue after final attempt
                    let mut queue = ack_queue.lock().unwrap();
                    queue.remove(&packet_id);
                }
            }
        }
    }
    if received_ack {
        Ok(())
    } else {
        Err(SessionError::AckError(AckError::new(
            "failed to retrieve ack ".to_string(),
        )))
    }
}

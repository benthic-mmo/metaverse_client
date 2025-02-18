use actix::prelude::*;
use log::{error, info};
use metaverse_messages::models::client_update_data::ClientUpdateData;
use metaverse_messages::models::packet::{MessageType, Packet};
use metaverse_messages::models::packet_ack::PacketAck;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::os::unix::net::UnixDatagram;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::Mutex;
use tokio::net::UdpSocket;
use tokio::sync::{oneshot, Notify};
use tokio::time::Duration;
use uuid::Uuid;

use super::errors::{AckError, SessionError};

#[derive(Debug)]
pub struct Mailbox {
    pub client_socket: u16,
    pub outgoing_socket: Option<OutgoingSocket>,

    pub ack_queue: Arc<Mutex<HashMap<u32, oneshot::Sender<()>>>>,
    pub update_stream: Arc<Mutex<Vec<ClientUpdateData>>>,

    pub packet_sequence_number: Arc<Mutex<u32>>,
    pub state: Arc<Mutex<ServerState>>,
    pub notify: Arc<Notify>,
    pub session: Option<Session>,
}

#[derive(Debug, Message)]
#[rtype(result = "()")]
pub struct Session {
    pub url: String,
    pub server_socket: u16,
    pub agent_id: Uuid,
    pub session_id: Uuid,
    pub socket: Option<Arc<UdpSocket>>,
}

#[derive(Debug, Message)]
#[rtype(result = "()")]
pub struct OutgoingSocket {
    pub socket_path: PathBuf,
}

// this is a debug method that triggers a message send from the mailbox
#[derive(Debug, Message, Serialize, Deserialize)]
#[rtype(result = "()")]
pub struct TriggerSend {
    pub message_type: String,
    pub encoding: String,
    pub sequence_number: u16,
    pub total_packet_number: u16,
    pub message: String,
}
impl TriggerSend {
    /// Convert the struct into bytes using JSON serialization
    pub fn as_bytes(&self) -> Vec<u8> {
        serde_json::to_vec(self).expect("Failed to serialize TriggerSend")
    }

    /// Convert bytes back into a `TriggerSend` struct
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, serde_json::Error> {
        serde_json::from_slice(bytes)
    }
}

const ACK_ATTEMPTS: i8 = 3;
const ACK_TIMEOUT: Duration = Duration::from_secs(1);

impl Mailbox {
    // this is for reading packets coming from the external server
    async fn start_udp_read(
        ack_queue: Arc<Mutex<HashMap<u32, oneshot::Sender<()>>>>,
        command_queue: Arc<Mutex<HashMap<u32, oneshot::Sender<()>>>>,
        data_queue: Arc<Mutex<HashMap<u32, oneshot::Sender<()>>>>,
        event_queue: Arc<Mutex<HashMap<u32, oneshot::Sender<()>>>>,
        request_queue: Arc<Mutex<HashMap<u32, oneshot::Sender<()>>>>,
        error_queue: Arc<Mutex<HashMap<u32, oneshot::Sender<()>>>>,
        outgoing_queue: Arc<Mutex<HashMap<u32, oneshot::Sender<()>>>>,
        update_stream: Arc<Mutex<Vec<ClientUpdateData>>>,
        sock: Arc<UdpSocket>,
        mailbox_address: Addr<Mailbox>,
    ) {
        let mut buf = [0; 1024];
        loop {
            match sock.recv_from(&mut buf).await {
                Ok((size, addr)) => {
                    //info!("Received {} bytes from {:?}", size, addr);

                    let packet = match Packet::from_bytes(&buf[..size]) {
                        Ok(packet) => packet,
                        Err(_) => {
                            //error!("failed to parse packet: {:?}", e);
                            continue;
                        }
                    };
                    if packet.header.reliable {
                        match mailbox_address
                            .send(Packet::new_packet_ack(PacketAck {
                                packet_ids: vec![packet.header.sequence_number],
                            }))
                            .await
                        {
                            Ok(_) => println!("ack sent"),
                            Err(_) => println!("ack failed to send"),
                        };
                    }
                    match packet.body.message_type() {
                        MessageType::Login => {
                            packet
                                .body
                                .on_receive(ack_queue.clone(), update_stream.clone())
                                .await;
                        }
                        MessageType::Acknowledgment => {
                            packet
                                .body
                                .on_receive(ack_queue.clone(), update_stream.clone())
                                .await;
                        }
                        MessageType::Command => {
                            packet
                                .body
                                .on_receive(command_queue.clone(), update_stream.clone())
                                .await;
                        }
                        MessageType::Data => {
                            packet
                                .body
                                .on_receive(data_queue.clone(), update_stream.clone())
                                .await;
                        }
                        MessageType::Event => {
                            packet
                                .body
                                .on_receive(event_queue.clone(), update_stream.clone())
                                .await;
                        }
                        MessageType::Request => {
                            packet
                                .body
                                .on_receive(request_queue.clone(), update_stream.clone())
                                .await;
                        }
                        MessageType::Error => {
                            packet
                                .body
                                .on_receive(error_queue.clone(), update_stream.clone())
                                .await;
                        }
                        MessageType::Outgoing => {
                            packet
                                .body
                                .on_receive(outgoing_queue.clone(), update_stream.clone())
                                .await;
                        }
                    };
                    info!("packet received: {:?}", packet);
                }
                Err(e) => {
                    eprintln!("Failed to receive data: {}", e);
                    break;
                }
            }
        }
    }

    fn set_state(&mut self, new_state: ServerState, _ctx: &mut Context<Self>) {
        let state_clone = Arc::clone(&self.state);
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

#[derive(Debug, Clone, PartialEq)]
pub enum ServerState {
    Starting,
    Running,
    Stopping,
    Stopped,
}

impl Actor for Mailbox {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        info!("Actix Mailbox has started");

        self.set_state(ServerState::Running, ctx);
    }
}

impl Handler<TriggerSend> for Mailbox {
    type Result = ();
    fn handle(&mut self, msg: TriggerSend, _: &mut Self::Context) -> Self::Result {
        if let Some(outgoing_socket) = &self.outgoing_socket {
            let max_message_size = 1024;
            // json inflates the size of packages by adding braces and escape characters.
            // This shouldn't change too much unless we change the struct
            let json_overhead = 200;

            // Serialize the common fields (message_type, encoding, sequence_number)
            let message_type_len = msg.message_type.len();
            let encoding_len = msg.encoding.len();
            let sequence_number_len = std::mem::size_of::<u16>(); // 2 bytes for the sequence number
            let total_packet_number_len = std::mem::size_of::<u16>();

            // Calculate the maximum size available for the actual message content
            let available_size = max_message_size
                - (message_type_len
                    + encoding_len
                    + sequence_number_len
                    + total_packet_number_len
                    + json_overhead);

            // Ensure we have enough space for the actual message
            if available_size <= 0 {
                println!("Error: Message is too large to fit within the 1024 byte limit.");
                return;
            }

            // Split the message content if it's larger than the available size
            let message = msg.message;
            let message_bytes = message.as_bytes();
            let total_chunks = (message_bytes.len() + available_size - 1) / available_size;

            // Loop through each chunk and send it
            for chunk_index in 0..total_chunks {
                let start = chunk_index * available_size;
                let end = usize::min(start + available_size, message_bytes.len());
                let chunk = &message_bytes[start..end];

                // Increment the sequence number for each chunk
                let sequence_number = msg.sequence_number + chunk_index as u16;

                // Create a new message with the chunked data
                let chunked_message = TriggerSend {
                    message_type: msg.message_type.clone(),
                    encoding: msg.encoding.clone(),
                    sequence_number,
                    total_packet_number: total_chunks as u16, // Add total number of chunks
                    message: String::from_utf8_lossy(chunk).to_string(),
                };

                // Send the chunk using the UnixDatagram socket
                let client_socket = UnixDatagram::unbound().unwrap();
                if let Err(e) =
                    client_socket.send_to(&chunked_message.as_bytes(), &outgoing_socket.socket_path)
                {
                    error!(
                        "Error sending chunk {} of {} from mailbox: {:?}",
                        sequence_number, total_chunks, e
                    )
                }
            }
        }
    }
}

impl Handler<OutgoingSocket> for Mailbox {
    type Result = ();
    fn handle(&mut self, msg: OutgoingSocket, _: &mut Self::Context) -> Self::Result {
        self.outgoing_socket = Some(msg);
    }
}

// set the session to initialized.
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
                let addr = format!("127.0.0.1:{}", self.client_socket);

                let addr_clone = addr.clone();
                let ack_queue_clone = self.ack_queue.clone();
                let command_queue_clone = self.ack_queue.clone();
                let data_queue_clone = self.ack_queue.clone();
                let event_queue_clone = self.ack_queue.clone();
                let request_queue_clone = self.ack_queue.clone();
                let error_queue_clone = self.ack_queue.clone();
                let outgoing_queue_clone = self.ack_queue.clone();
                let update_stream_clone = self.update_stream.clone();
                let mailbox_addr = ctx.address();

                println!("session established, starting UDP processing");

                let fut = async move {
                    match UdpSocket::bind(&addr).await {
                        Ok(sock) => {
                            println!("Successfully bound to {}", &addr);

                            let sock = Arc::new(sock);

                            // Spawn a new Tokio task for reading from the socket
                            tokio::spawn(Mailbox::start_udp_read(
                                ack_queue_clone,
                                command_queue_clone,
                                data_queue_clone,
                                event_queue_clone,
                                request_queue_clone,
                                error_queue_clone,
                                outgoing_queue_clone,
                                update_stream_clone,
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

impl Handler<Packet> for Mailbox {
    type Result = ();
    fn handle(&mut self, mut msg: Packet, ctx: &mut Self::Context) -> Self::Result {
        if let Some(ref session) = self.session {
            let addr = format!("{}:{}", session.url, session.server_socket);
            {
                let sequence_number = self.packet_sequence_number.lock().unwrap();
                msg.header.sequence_number = *sequence_number;
                println!("PACKET NUMBER IS: {}", *sequence_number);
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
                    info!("sent data to {}", addr)
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

    println!("PACKET IS: {:?}", packet);
    println!("SENDING ACK FOR PACKET ID: {}", packet_id);

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

            println!("QUEUE IS: {:?}", queue);
        }
        // Send the packet

        let data = packet_clone.to_bytes().clone();
        let addr_clone = addr.clone();
        let sock_clone = socket.clone();
        if let Err(e) = sock_clone.send_to(&data, addr_clone).await {
            error!("Failed to send data: {}", e);
        }

        tokio::select! {
            _ = rx => {
                println!("RECEIVED ACK FOR {}", packet_id);
                received_ack = true;
            },
            _ = tokio::time::sleep(ACK_TIMEOUT) => {
                println!("Attempt {} failed to receive acknowledgment", attempts);
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

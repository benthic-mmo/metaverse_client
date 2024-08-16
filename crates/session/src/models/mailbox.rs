use actix::prelude::*;
use futures::future::BoxFuture;
use log::{error, info};
use metaverse_messages::models::client_update_data::{ClientUpdateContent, ClientUpdateData, DataContent};
use metaverse_messages::models::packet::{MessageType, Packet};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::net::UdpSocket;
use tokio::sync::oneshot;
use tokio::sync::Mutex;
use tokio::time::sleep;

pub struct Mailbox {
    pub socket: Option<Arc<UdpSocket>>,
    pub url: String,
    pub server_socket: u16,
    pub client_socket: u16,

    pub ack_queue: Arc<Mutex<HashMap<u32, oneshot::Sender<()>>>>,
    pub request_queue: Arc<Mutex<HashMap<u32, oneshot::Sender<()>>>>,
    pub event_queue: Arc<Mutex<HashMap<u32, oneshot::Sender<()>>>>,
    pub command_queue: Arc<Mutex<HashMap<u32, oneshot::Sender<()>>>>,
    pub error_queue: Arc<Mutex<HashMap<u32, oneshot::Sender<()>>>>,
    pub data_queue: Arc<Mutex<HashMap<u32, oneshot::Sender<()>>>>,

    pub update_stream: Arc<Mutex<Vec<ClientUpdateData>>>,
}

impl Mailbox {
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
    ) {
        let mut buf = [0; 1024];
        loop {
            match sock.recv_from(&mut buf).await {
                Ok((size, addr)) => {
                    info!("Received {} bytes from {:?}", size, addr);

                    let packet = match Packet::from_bytes(&buf[..size]) {
                        Ok(packet) => packet,
                        Err(e) => {
                            error!("failed to parse packet: {:?}", e);
                            continue;
                        }
                    };
                    match packet.body.message_type() {
                        MessageType::Acknowledgment => {
                            packet.body.on_receive(ack_queue.clone(), update_stream.clone()).await;
                        }
                        MessageType::Command => {
                            packet.body.on_receive(command_queue.clone(), update_stream.clone()).await;
                        }
                        MessageType::Data => {
                            packet.body.on_receive(data_queue.clone(), update_stream.clone()).await;
                        }
                        MessageType::Event => {
                            packet.body.on_receive(event_queue.clone(), update_stream.clone()).await;
                        }
                        MessageType::Request => {
                            packet.body.on_receive(request_queue.clone(), update_stream.clone()).await;
                        }
                        MessageType::Error => {
                            packet.body.on_receive(error_queue.clone(), update_stream.clone()).await;
                        }
                        MessageType::Outgoing => {
                            packet.body.on_receive(outgoing_queue.clone(), update_stream.clone()).await;
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
}

impl Actor for Mailbox {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        info!("Actix Mailbox has started");

        let addr = format!("127.0.0.1:{}", self.client_socket.clone());
        let addr_clone = addr.clone();
        let ack_queue_clone = self.ack_queue.clone();
        let command_queue_clone = self.ack_queue.clone();
        let data_queue_clone = self.ack_queue.clone();
        let event_queue_clone = self.ack_queue.clone();
        let request_queue_clone = self.ack_queue.clone();
        let error_queue_clone = self.ack_queue.clone();
        let outgoing_queue_clone = self.ack_queue.clone();
        let update_stream_clone = self.update_stream.clone();
        let fut = async move {
            match UdpSocket::bind(&addr).await {
                Ok(sock) => {
                    println!("Successfully bound to {}", &addr_clone);

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
                    ));

                    Ok(sock) // Return the socket wrapped in Arc
                }
                Err(e) => {
                    error!("Failed to bind to {}: {}", &addr_clone, e);
                    Err(e)
                }
            }
        };

        ctx.spawn(fut.into_actor(self).map(|result, act, _| match result {
            Ok(sock) => {
                act.socket = Some(sock);
            }
            Err(_) => {
                panic!("Socket binding failed");
            }
        }));
    }
}

impl Handler<Packet> for Mailbox {
    type Result = ();
    fn handle(&mut self, msg: Packet, ctx: &mut Self::Context) -> Self::Result {
        if let Some(ref sock) = self.socket {
            let addr = format!("{}:{}", self.url, self.server_socket);
            let data = msg.to_bytes().clone();
            let socket_clone = sock.clone();
            let fut = async move {
                if let Err(e) = socket_clone.send_to(&data, &addr).await {
                    error!("Failed to send data: {}", e);
                }
                info!("sent data to {}", addr)
            };
            ctx.spawn(fut.into_actor(self));
        }
    }
}

pub trait AllowAcks {
    fn send_with_ack(
        &self,
        packet: Packet,
        timeout: Duration,
        max_attempts: i8,
        update_stream: Arc<Mutex<Vec<ClientUpdateData>>>
    ) -> BoxFuture<'static, Result<(), String>>;
}

impl AllowAcks for Addr<Mailbox> {
    fn send_with_ack(
        &self,
        packet: Packet,
        timeout: Duration,
        max_attempts: i8,
        update_stream: Arc<Mutex<Vec<ClientUpdateData>>>,
    ) -> BoxFuture<'static, Result<(), String>> {
        let addr = self.clone();
        Box::pin(async move {
            let mut attempts = 0;
            let mut received_ack = false;

            while attempts < max_attempts && !received_ack {
                let (tx, rx) = oneshot::channel();
                let packet_clone = packet.clone();
                let packet_id = packet.header.sequence_number;

                // Get the ack queue
                let ack_queue = match addr.send(GetAckQueue).await {
                    Ok(queue) => queue,
                    Err(e) => return Err(format!("Failed to get ack queue: {}", e)),
                };

                // these brackets make a new scope so the queue is opened and closed inside them
                {
                    let mut queue = ack_queue.lock().await;
                    queue.insert(packet_id, tx);
                }
                // Send the packet
                addr.do_send(packet_clone);

                tokio::select! {
                    _ = rx => {
                        received_ack = true;
                    },
                    _ = sleep(timeout) => {
                        println!("Attempt {} failed to receive acknowledgment", attempts);
                        if !received_ack {
                            {
                                let mut queue = ack_queue.lock().await;
                                queue.remove(&1);
                            }
                        }
                        attempts += 1;
                    }
                }
            }
            if received_ack {
                let mut client = update_stream.lock().await;
                client.push(ClientUpdateData {
                        content: ClientUpdateContent::Data(DataContent {
                            content: "hello world".to_string(),
                        }),
                });
                Ok(())
            } else {
                Err("Failed to receive acknowledgment".into())
            }
        })
    }
}
#[derive(Message)]
#[rtype(result = "Arc<Mutex<HashMap<u32, oneshot::Sender<()>>>>")]
struct GetAckQueue;

impl Handler<GetAckQueue> for Mailbox {
    type Result = Arc<Mutex<HashMap<u32, oneshot::Sender<()>>>>;

    fn handle(&mut self, _msg: GetAckQueue, _ctx: &mut Self::Context) -> Self::Result {
        self.ack_queue.clone()
    }
}

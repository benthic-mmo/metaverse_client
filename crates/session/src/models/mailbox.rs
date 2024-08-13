use actix::prelude::*;
use log::{error, info};
use metaverse_messages::models::packet::Packet;
use std::sync::Arc;
use tokio::net::UdpSocket;

pub struct Mailbox {
    pub socket: Option<Arc<UdpSocket>>,
    pub url: String,
    pub server_socket: u16,
    pub client_socket: u16,
}

impl Mailbox {
    async fn start_udp_read(sock: Arc<UdpSocket>) {
        let mut buf = [0; 1024];
        loop {
            info!("udp read is running");
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
        let fut = async move {
            match UdpSocket::bind(&addr).await {
                Ok(sock) => {
                    println!("Successfully bound to {}", &addr_clone);

                    let sock = Arc::new(sock);

                    // Spawn a new Tokio task for reading from the socket
                    tokio::spawn(Mailbox::start_udp_read(sock.clone()));

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

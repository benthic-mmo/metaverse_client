use crate::session::{Mailbox, Ping, SendAckList};
use actix::Addr;
use log::{error, warn};
use metaverse_messages::packet::{message::UIMessage, packet::Packet, packet_types::PacketType};
use std::sync::Mutex;
use std::{collections::HashSet, sync::Arc};
use tokio::net::UdpSocket;
/// This file is for handling the UDP messages that are sent from the server to the client.

impl Mailbox {
    /// Start_udp_read is for reading packets coming from the external server
    pub async fn start_udp_read(
        ack_queue: Arc<Mutex<HashSet<u32>>>,
        sock: Arc<UdpSocket>,
        mailbox_address: Addr<Mailbox>,
    ) {
        let mut buf = [0; 1500];

        loop {
            match sock.recv_from(&mut buf).await {
                Ok((size, _addr)) => {
                    let packet = match Packet::from_bytes(&buf[..size]) {
                        Ok(packet) => packet,
                        Err(e) => {
                            println!("{:?}", e);
                            //this currently has a lot of packets that don't parse. If this error
                            //were to be visible it would be a constant spam. Someday this will not
                            //be the case.
                            continue;
                        }
                    };

                    // if the incoming packet's header is reliable, add it to the ack list, and then trigger a send
                    if packet.header.reliable {
                        {
                            ack_queue
                                .lock()
                                .unwrap()
                                .insert(packet.header.sequence_number);
                        }
                        if let Err(e) = mailbox_address.send(SendAckList {}).await {
                            warn!("Failed to send ack list {:?}", e)
                        }
                    }

                    match &packet.body {
                        PacketType::StartPingCheck(data) => {
                            if let Err(e) = mailbox_address
                                .send(Ping {
                                    ping_id: data.ping_id,
                                })
                                .await
                            {
                                warn!("failed to handle pong {:?}", e)
                            };
                        }
                        PacketType::RegionHandshake(data) => {
                            error!("{:?}", data);
                            if let Err(e) = mailbox_address.send(*data.clone()).await {
                                error!("error: {:?}", e)
                            }
                        }
                        PacketType::DisableSimulator(_) => {
                            warn!("Simulator shutting down...");
                            if let Err(e) = mailbox_address
                                .send(UIMessage::new_disable_simulator())
                                .await
                            {
                                warn!("failed to send to ui: {:?}", e)
                            }
                            break;
                        }
                        PacketType::ObjectUpdate(data) => {
                            if let Err(e) = mailbox_address.send(*data.clone()).await {
                                error!("Failed to handle ObjectUpdate {:?}", e)
                            };
                        }
                        #[cfg(feature = "environment")]
                        PacketType::LayerData(data) => {
                            if let Err(e) = mailbox_address.send(*data.clone()).await {
                                error!("Failed to handle AgentWearablesUpdate {:?}", e)
                            };
                        }
                        // Send UI packets to the UI as UiMessages.
                        PacketType::ChatFromSimulator(data) => {
                            if let Err(e) = mailbox_address
                                .send(UIMessage::new_chat_from_simulator(*data.clone()))
                                .await
                            {
                                error!("Failed to handle chatfromsimulator{:?}", e)
                            }
                        }
                        other => {
                            println!("{:?}", other);
                        }
                    }
                }
                Err(e) => {
                    error!("Failed to receive data: {}", e);
                    break;
                }
            }
        }
    }
}

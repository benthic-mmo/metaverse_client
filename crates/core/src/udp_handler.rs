use actix::Addr;
use std::{collections::HashMap, sync::Arc};
use tokio::net::UdpSocket;

use log::{error, warn};
use metaverse_messages::{
    core::packet_ack::PacketAck,
    packet::{packet::Packet, packet_types::PacketType},
    ui::ui_events::UiEventTypes,
};
use std::sync::Mutex;
use tokio::sync::oneshot;

use crate::core::session::{Mailbox, Ping, RegionHandshakeMessage, UiMessage};
impl Mailbox {
    /// Start_udp_read is for reading packets coming from the external server
    pub async fn start_udp_read(
        ack_queue: Arc<Mutex<HashMap<u32, oneshot::Sender<()>>>>,
        sock: Arc<UdpSocket>,
        mailbox_address: Addr<Mailbox>,
    ) {
        let mut buf = [0; 1500];

        loop {
            match sock.recv_from(&mut buf).await {
                Ok((size, _addr)) => {
                    //info!("Received {} bytes from {:?}", size, addr);

                    let packet = match Packet::from_bytes(&buf[..size]) {
                        Ok(packet) => packet,
                        Err(_) => {
                            //                            println!("{:?}", e);
                            continue;
                        }
                    };
                    if packet.header.reliable {
                        if let Err(e) = mailbox_address
                            .send(Packet::new_packet_ack(PacketAck {
                                packet_ids: vec![packet.header.sequence_number],
                            }))
                            .await
                        {
                            warn!("Ack failed to send {:?}", e)
                        };
                    }

                    match &packet.body {
                        PacketType::PacketAck(data) => {
                            let mut queue = ack_queue.lock().unwrap();
                            for id in data.packet_ids.clone() {
                                if let Some(sender) = queue.remove(&id) {
                                    let _ = sender.send(());
                                }
                            }
                        }
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
                        PacketType::RegionHandshake(_) => {
                            if let Err(e) = mailbox_address.send(RegionHandshakeMessage {}).await {
                                error!("error: {:?}", e)
                            }
                        }
                        PacketType::DisableSimulator(_) => {
                            warn!("Simulator shutting down...");
                            if let Err(e) = mailbox_address
                                .send(UiMessage::new(
                                    UiEventTypes::DisableSimulatorEvent {},
                                    vec![],
                                ))
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
                        _ => {}
                    }
                    if UiEventTypes::None != packet.body.ui_event() {
                        if let Err(e) = mailbox_address
                            .send(UiMessage::new(
                                packet.body.ui_event(),
                                packet.body.to_bytes(),
                            ))
                            .await
                        {
                            warn!("failed to send to ui: {:?}", e)
                        };
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

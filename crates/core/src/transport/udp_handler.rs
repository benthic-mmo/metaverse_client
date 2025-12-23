use crate::avatar::HandleNewAvatarAppearance;
use crate::environment::HandleLayerData;
use crate::objects::{
    HandleImprovedTerseObjectUpdate, HandleObjectUpdate, HandleObjectUpdateCached,
};
use crate::session::{
    AddToAckList, HandlePacketAck, HandlePing, HandleRegionHandshake, Mailbox, SendUIMessage,
};
use actix::Addr;
use log::{error, warn};
use metaverse_messages::packet::{message::UIMessage, packet::Packet, packet_types::PacketType};
use std::sync::Arc;
use tokio::net::UdpSocket;

impl Mailbox {
    /// Start_udp_read is for reading packets coming from the external server
    pub async fn start_udp_read(sock: Arc<UdpSocket>, mailbox_address: Addr<Mailbox>) {
        let mut buf = [0; 1500];

        loop {
            match sock.recv_from(&mut buf).await {
                Ok((size, _addr)) => {
                    let packet = match Packet::from_bytes(&buf[..size]) {
                        Ok(packet) => packet,
                        Err(e) => {
                            println!("failed to parse: {:?}", e);
                            continue;
                        }
                    };
                    // if the incoming packet's header is reliable, add it to the ack list, and then trigger a send
                    if packet.header.reliable {
                        if let Err(e) = mailbox_address
                            .send(AddToAckList {
                                id: packet.header.sequence_number,
                            })
                            .await
                        {
                            warn!("Failed to send ping: {:?}", e)
                        }
                    }

                    match &packet.body {
                        PacketType::PacketAck(data) => {
                            if let Err(e) = mailbox_address
                                .send(HandlePacketAck {
                                    packet_ack: *data.clone(),
                                })
                                .await
                            {
                                error!("Failed to handle PacketAck {:?}", e)
                            }
                        }
                        PacketType::StartPingCheck(data) => {
                            if let Err(e) = mailbox_address
                                .send(HandlePing {
                                    ping_id: data.ping_id,
                                })
                                .await
                            {
                                warn!("failed to handle pong {:?}", e)
                            };
                        }
                        PacketType::RegionHandshake(data) => {
                            if let Err(e) = mailbox_address
                                .send(HandleRegionHandshake {
                                    region_handshake: *data.clone(),
                                })
                                .await
                            {
                                error!("Failed to handle RegionHandshake {:?}", e)
                            }
                        }
                        PacketType::DisableSimulator(_) => {
                            warn!("Simulator shutting down...");
                            if let Err(e) = mailbox_address
                                .send(SendUIMessage {
                                    ui_message: UIMessage::new_disable_simulator(),
                                })
                                .await
                            {
                                warn!("failed to send to ui: {:?}", e)
                            }
                            break;
                        }
                        PacketType::ObjectUpdate(data) => {
                            if let Err(e) = mailbox_address
                                .send(HandleObjectUpdate {
                                    object_type: data.pcode,
                                    full_id: data.full_id,
                                    parent_id: Some(data.parent_id),
                                    local_id: data.id,
                                    name_value: Some(data.name_value.clone()),
                                    position: data.motion_data.position,
                                    extra_params: data.extra_params.clone(),
                                    rotation: data.motion_data.rotation,
                                    scale: data.scale,
                                    parent: Some(data.parent_id),
                                })
                                .await
                            {
                                error!("Failed to handle ObjectUpdate {:?}", e)
                            };
                        }
                        PacketType::ObjectUpdateCached(data) => {
                            if let Err(e) = mailbox_address
                                .send(HandleObjectUpdateCached {
                                    object_update_cached: *data.clone(),
                                })
                                .await
                            {
                                error!("Failed to handle ObjectUpdateCached {:?}", e)
                            };
                        }
                        PacketType::ImprovedTerseObjectUpdate(data) => {
                            if let Err(e) = mailbox_address
                                .send(HandleImprovedTerseObjectUpdate {
                                    improved_terse_object_update: *data.clone(),
                                })
                                .await
                            {
                                error!("Failed to handle TerseObjectUpdate {:?}", e)
                            };
                        }
                        PacketType::ObjectUpdateCompressed(data) => {
                            for object in data.object_data.clone() {
                                if let Err(e) = mailbox_address
                                    .send(HandleObjectUpdate {
                                        object_type: object.pcode,
                                        full_id: object.full_id,
                                        parent_id: object.parent_id,
                                        local_id: object.local_id,
                                        name_value: object.name_values,
                                        position: object.position,
                                        extra_params: object.extra_params,
                                        rotation: object.rotation,
                                        scale: object.scale,
                                        parent: object.parent_id,
                                    })
                                    .await
                                {
                                    error!("Failed to handle ObjectUpdateCompressed {:?}", e)
                                };
                            }
                        }
                        #[cfg(feature = "environment")]
                        PacketType::LayerData(data) => {
                            if let Err(e) = mailbox_address
                                .send(HandleLayerData {
                                    layer_data: *data.clone(),
                                })
                                .await
                            {
                                error!("Failed to handle LayerData {:?}", e)
                            };
                        }
                        // Send UI packets to the UI as UiMessages.
                        PacketType::ChatFromSimulator(data) => {
                            if let Err(e) = mailbox_address
                                .send(SendUIMessage {
                                    ui_message: UIMessage::new_chat_from_simulator(*data.clone()),
                                })
                                .await
                            {
                                error!("Failed to handle chatfromsimulator{:?}", e)
                            }
                        }
                        PacketType::AvatarAppearance(data) => {
                            if let Err(e) = mailbox_address
                                .send(HandleNewAvatarAppearance {
                                    avatar_appearance: *data.clone(),
                                })
                                .await
                            {
                                error!("Failed to handle AvatarAppearance {:?}", e)
                            };
                        }
                        other => {
                            println!("unhandled packet: {:?}", other);
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

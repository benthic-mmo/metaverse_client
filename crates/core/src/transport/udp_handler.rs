use crate::avatar::{HandleNewAvatarAnimation, HandleNewAvatarAppearance};
use crate::environment::{HandleLayerData, HandleSimulatorViewerTimeMessage};
use crate::objects::{
    AttachmentObjectUpdate, AvatarObjectUpdate, HandleImprovedTerseObjectUpdate,
    HandleObjectUpdateCached, MeshObjectUpdate, ParametricPrimObjectUpdate,
};
use crate::session::{
    AddToAckList, HandlePacketAck, HandlePing, HandleRegionHandshake, Mailbox, SendUIMessage,
};
use actix::{Addr, Message};
use benthic_protocol::errors::SessionError;
use benthic_protocol::messages::ui::chat_from_simulator::ChatFromSimulator;
use benthic_protocol::messages::ui::ui_messages::UIMessage;
use benthic_protocol::objects::{AttachmentObjectData, MeshObjectData, ParametricPrimData};
use log::{error, warn};
use metaverse_messages::http::scene::SculptType;
use metaverse_messages::packet::{packet_protocol::Packet, packet_types::PacketType};
use metaverse_messages::udp::object::object_update::AttachItem;
use metaverse_messages::udp::object::object_update::ExtraParams::{self, Flexi};
use metaverse_messages::udp::object::object_update_compressed::ObjectUpdateCompressed;
use metaverse_messages::udp::object::util::ObjectUpdateData;
use metaverse_messages::utils::object_types::ObjectType;
use metaverse_objects::errors::ObjectUpdateError;
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
                            //use std::fs::File;
                            //use std::io::Write;
                            //let mut file = File::create("foo.txt").unwrap();
                            //file.write_all(&buf[..size]).unwrap();
                            warn!("failed to parse: {:?}", e);
                            continue;
                        }
                    };
                    // if the incoming packet's header is reliable, add it to the ack list, and then trigger a send
                    if packet.header.reliable
                        && let Err(e) = mailbox_address
                            .send(AddToAckList {
                                id: packet.header.sequence_number,
                            })
                            .await
                    {
                        warn!("Failed to send ping: {:?}", e)
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
                            if let Err(e) =
                                classify_object_update(*data.clone(), mailbox_address.clone()).await
                            {
                                error!("Failed to handle object update: {:?}", e);
                            }
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
                                if let Err(e) =
                                    classify_object_update(object, mailbox_address.clone()).await
                                {
                                    error!("Failed to handle object update compressed: {:?}", e);
                                }
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
                                    ui_message: UIMessage::new_chat_from_simulator(
                                        ChatFromSimulator {
                                            from_name: data.from_name.clone(),
                                            audible: data.audible.clone(),
                                            chat_type: data.chat_type.clone(),
                                            source_id: data.source_id,
                                            owner_id: data.owner_id,
                                            position: data.position,
                                            source_type: data.source_type.clone(),
                                            message: data.message.clone(),
                                        },
                                    ),
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
                        PacketType::AvatarAnimation(data) => {
                            if let Err(e) = mailbox_address
                                .send(HandleNewAvatarAnimation {
                                    avatar_animation: *data.clone(),
                                    retries: 0,
                                })
                                .await
                            {
                                error!("Failed to handle AvatarAppearance {:?}", e)
                            };
                        }
                        PacketType::SimulatorViewerTimeMessage(data) => {
                            if let Err(e) = mailbox_address
                                .send(HandleSimulatorViewerTimeMessage {
                                    seconds_since_start: data.seconds_since_start,
                                    sun_phase: data.sun_phase,
                                    seconds_per_day: data.seconds_per_day,
                                    seconds_per_year: data.seconds_per_year,
                                })
                                .await
                            {
                                error!("Failed to handle SimulatorViewerTimeMessage {:?}", e)
                            };
                        }

                        other => {
                            warn!("unhandled packet: {:?}", other);
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

async fn classify_object_update(
    object: impl ObjectUpdateData,
    addr: Addr<Mailbox>,
) -> Result<(), ObjectUpdateError> {
    let action = match object.object_type() {
        ObjectType::Prim => {
            if let Some(name) = object.name_value().as_ref()
                && let Ok(attachment) = AttachItem::parse_attach_item(name)
            {
                addr.do_send(AttachmentObjectUpdate(AttachmentObjectData {
                    parent_id: object.parent_id(),
                }));
            }
            if let Some(params) = object.extra_params() {
                for param in params {
                    match param {
                        ExtraParams::Sculpt(sculpt) => match sculpt.sculpt_type {
                            SculptType::Mesh {} => addr.do_send(MeshObjectUpdate(MeshObjectData {
                                object_type: object.object_type(),
                                full_id: object.full_id(),
                                parent: object.parent_id(),
                                local_id: object.local_id(),
                                position: object.position(),
                                sculpt_id: sculpt.texture_id,
                                rotation: object.rotation(),
                                scale: object.scale(),
                                texture: object.texture().clone(),
                                crc: object.crc(),
                                region_id: "".to_string(),
                            })),
                            SculptType::Plane
                            | SculptType::Sphere
                            | SculptType::Torus
                            | SculptType::Cylinder => {
                                if let Some(path) = object.sculpt_path() {
                                    addr.do_send(ParametricPrimObjectUpdate(ParametricPrimData {
                                        full_id: object.full_id(),
                                        local_id: object.local_id(),
                                        scale: object.scale(),
                                        position: object.position(),
                                        rotation: object.rotation(),
                                        parent: object.parent_id(),
                                        texture: object.texture().clone(),
                                        path_data: path,
                                    }))
                                } else {
                                    error!("No sculpt path found in a parametric prim.")
                                }
                            }
                            _ => Err(ObjectUpdateError::Unimplemented {
                                feature: "Unimplemented Sculpt Type".to_string(),
                            })?,
                        },
                        _ => Err(ObjectUpdateError::Unimplemented {
                            feature: "Unimplented Parameter Type".to_string(),
                        })?,
                    }
                }
            };
        }
        ObjectType::Avatar => addr.do_send(AvatarObjectUpdate {
            id: object.full_id(),
            position: object.position(),
            retry_count: 0,
        }),
        _ => {}
    };
    Ok(())
}

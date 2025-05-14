use actix::{Addr, Context};
use std::{collections::HashMap, sync::Arc};
use tokio::net::UdpSocket;

#[cfg(feature = "agent")]
use metaverse_agent::{
    avatar_appearance_handler::{parse_texture_data, parse_visual_param_data},
    object_update_handler::handle_object_update,
};

#[cfg(feature = "environment")]
use metaverse_environment::layer_handler::{PatchData, PatchLayer, parse_layer_data};

use log::{error, info, warn};
use metaverse_messages::{
    core::{object_update::ObjectType, packet_ack::PacketAck},
    packet::{packet::Packet, packet_types::PacketType},
    ui::ui_events::UiEventTypes,
};
use std::sync::Mutex;
use tokio::sync::oneshot;

use crate::core::{Mailbox, Ping, RegionHandshakeMessage, ServerState, UiMessage};
impl Mailbox {
    /// Start_udp_read is for reading packets coming from the external server
    pub async fn start_udp_read(
        ack_queue: Arc<Mutex<HashMap<u32, oneshot::Sender<()>>>>,
        sock: Arc<UdpSocket>,
        mailbox_address: Addr<Mailbox>,
    ) {
        let mut buf = [0; 1500];

        #[cfg(feature = "environment")]
        let mut patch_queue = HashMap::new();

        #[cfg(feature = "environment")]
        let mut total_patches = HashMap::new();

        loop {
            match sock.recv_from(&mut buf).await {
                Ok((size, _addr)) => {
                    //info!("Received {} bytes from {:?}", size, addr);

                    let packet = match Packet::from_bytes(&buf[..size]) {
                        Ok(packet) => packet,
                        Err(_) => {
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
                        PacketType::ObjectUpdate(data) => match data.pcode {
                            ObjectType::Tree
                            | ObjectType::Grass
                            | ObjectType::Prim
                            | ObjectType::Unknown
                            | ObjectType::ParticleSystem
                            | ObjectType::NewTree
                            | ObjectType::None => {
                                #[cfg(feature = "environment")]
                                info!("Received environment data");
                            }
                            ObjectType::Avatar => {
                                #[cfg(feature = "agent")]
                                if let Err(e) = handle_object_update(data) {
                                    warn!("Error handling avatar {:?}", e);
                                };
                            }
                        },
                        #[cfg(feature = "environment")]
                        PacketType::LayerData(data) => {
                            if let Ok(patch_data) = parse_layer_data(data) {
                                match patch_data {
                                    PatchLayer::Land(patches) => {
                                        for land in patches {
                                            total_patches
                                                .insert(land.terrain_header.location, land.clone());
                                            let mut layer_updates = land.generate_ui_event(
                                                &mut patch_queue,
                                                &total_patches,
                                            );
                                            let queue_save = patch_queue.clone();
                                            for (_location, land) in queue_save {
                                                layer_updates.extend(land.generate_ui_event(
                                                    &mut patch_queue,
                                                    &total_patches,
                                                ));
                                            }
                                            for layer in layer_updates {
                                                if let Err(e) = mailbox_address
                                                    .send(UiMessage::new(
                                                        UiEventTypes::LayerUpdateEvent,
                                                        layer.to_bytes(),
                                                    ))
                                                    .await
                                                {
                                                    println!(
                                                        "Failed to send LayerUpdate event to UI {:?}",
                                                        e
                                                    );
                                                }
                                            }
                                        }
                                    }
                                    PatchLayer::Wind(_patches) => {}
                                    PatchLayer::Water(_patches) => {}
                                    PatchLayer::Cloud(_patches) => {}
                                }
                            }
                        }
                        #[cfg(feature = "agent")]
                        PacketType::AvatarAppearance(data) => {
                            parse_texture_data(&data.texture_data);
                            parse_visual_param_data(&data.visual_param_data);
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

    pub fn set_state(&mut self, new_state: ServerState, _ctx: &mut Context<Self>) {
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

use crate::actors::DEFAULT_MAX_RETRIES;
use crate::actors::InteractionActor;
use crate::actors::SessionActor;
use crate::transport::udp_handler::start_udp_read;
use crate::ui_interactions::SendUIMessage;
use actix::prelude::*;
use benthic_protocol::messages::ui_protocol::UIMessage;
use benthic_protocol::messages::world::water_update::WaterUpdate;
use benthic_protocol::objects::types::RetriableData;
use benthic_protocol::session::types::Session;
use log::{error, info, warn};
use metaverse_avatar::avatar::Avatar;
use metaverse_environment::land::Land;
use metaverse_messages::{
    http::capabilities::Capability,
    packet::packet_protocol::Packet,
    udp::core::{
        complete_ping_check::CompletePingCheck, packet_ack::PacketAck,
        region_handshake::RegionHandshake, region_handshake_reply::RegionHandshakeReply,
    },
};
use metaverse_store::initialize_sqlite::Inventory;
use rand::RngExt;
use rgb::Rgba;
use std::any::type_name;
use std::net::IpAddr;
use std::sync::Arc;
use std::time;
use tokio::{net::UdpSocket, time::Duration};

/// Message used for initializing the session
///
/// # Cause
/// [`HandleUIResponse`]
///
/// # Effects
/// Spawns UDP handlers and initializes session
#[derive(Debug, Message)]
#[rtype(result = "()")]
pub struct StartSession {
    pub local_ip: IpAddr,
}
impl Handler<StartSession> for InteractionActor {
    type Result = ();
    fn handle(&mut self, msg: StartSession, ctx: &mut Self::Context) -> Self::Result {
        // if the session doesn't already have a UDP socket to watch, create one
        if self.socket.is_none() {
            let addr = format!("{}:{}", msg.local_ip, self.client_socket);

            let addr_clone = addr.clone();
            let interaction_addr = ctx.address();
            let mailbox_addr = self.mailbox_addr.clone();

            info!("session established, starting UDP processing");

            let fut = async move {
                match UdpSocket::bind(&addr).await {
                    Ok(sock) => {
                        info!("Successfully bound to {}", addr);
                        let sock = Arc::new(sock);
                        // Spawn a new Tokio task for reading from the socket
                        tokio::spawn(start_udp_read(sock.clone(), interaction_addr, mailbox_addr));
                        Ok(sock) // Return the socket wrapped in Arc
                    }
                    Err(e) => {
                        error!("Failed to bind to {}: {}", addr_clone, e);
                        Err(e)
                    }
                }
            };

            // wait for the socket to be successfully bound and then assign it
            ctx.spawn(fut.into_actor(self).map(|result, act, _| match result {
                Ok(sock) => {
                    act.socket = Some(sock.clone());
                }
                Err(_) => {
                    panic!("Socket binding failed");
                }
            }));
        }
    }
}
#[derive(Debug, Message)]
#[rtype(result = "()")]
pub struct AttachSession {
    pub session: Session<Capability, Avatar, Land, Inventory>,
}

impl Handler<AttachSession> for SessionActor {
    type Result = ();
    fn handle(&mut self, msg: AttachSession, _ctx: &mut Self::Context) -> Self::Result {
        self.session = Some(msg.session);
    }
}

/// Handles incoming pings from the server
///
/// # Cause
/// - Received StartPingCheck packet from UDP socket
#[derive(Debug, Message)]
#[rtype(result = "()")]
pub struct HandlePing {
    /// The ID of the ping
    pub ping_id: u8,
}
impl Handler<HandlePing> for InteractionActor {
    type Result = ();
    fn handle(&mut self, msg: HandlePing, ctx: &mut Self::Context) -> Self::Result {
        ctx.address().do_send(OutgoingPacket {
            packet: Packet::new_complete_ping_check(CompletePingCheck {
                ping_id: msg.ping_id,
            }),
        });
        self.ping_info.ping_latency = time::Instant::now() - self.ping_info.last_ping;
    }
}

/// Message used for acknowledging reliable headers for incoming packets.
///
/// When a UDP packet is received from the server with a reliable header, its sequence number is
/// sent back to the server in a PacketAck, so the server knows that sequence number was received.
/// Then the ack queue is cleared, to prevent sending acks for packets that have already been
/// received.
///
/// # Cause
/// - [`AddToAckList`]
///
/// # Effects
/// - Dispatches a [`PacketAck`] packet to the server
#[derive(Debug, Message)]
#[rtype(result = "()")]
pub struct SendAckList {}
impl Handler<SendAckList> for InteractionActor {
    type Result = ();
    fn handle(&mut self, _: SendAckList, ctx: &mut Self::Context) -> Self::Result {
        let Some(address) = self.session_context.address.get().clone() else {
            return;
        };
        // send ack directly to the server
        if self.server_acks.is_empty() {
            return;
        }
        let packet_ids: Vec<u32> = self.server_acks.drain().collect();

        let addr = address.clone();
        let packet = Packet::new_packet_ack(PacketAck { packet_ids }).to_bytes();
        let sock_clone = self.socket.clone().unwrap();
        let ack_wait = async move {
            if let Err(e) = sock_clone.send_to(&packet, addr).await {
                error!("Failed to send ack: {:?}", e)
            };
        };
        ctx.spawn(ack_wait.into_actor(self));
    }
}

/// Message for updating the list of unacked packets sent from the core to the server
///
/// Messages marked as reliable sent from the core must be resent until the server replies with a
/// PacketAck message. This adds thoes packets to the ack list.
///
/// # Cause
/// - Received a reliable packet from UDP socket
#[derive(Debug, Message)]
#[rtype(result = "()")]
pub struct AddToAckList {
    /// The ID of the packet to be acked
    pub id: u32,
}
impl Handler<AddToAckList> for InteractionActor {
    type Result = ();
    fn handle(&mut self, msg: AddToAckList, ctx: &mut Self::Context) -> Self::Result {
        self.server_acks.insert(msg.id);
        ctx.address().do_send(SendAckList {});
    }
}

/// Message for determining if a packet should be resent
///
/// When an outgoing packet is labeled reliable, this message is used to determine if it should be
/// resent to the server. An [`OutgoingPacket`] packet is sent initially, followed by a brief
/// timeout. This allows the server enough time to respond with an ack. If an ack is not received,
/// the packet is not resent. If it isn't, the packet will be resent.
///
/// # Cause
/// - [`OutgoingPacket`] on a reliable packet
///
/// # Effect
/// - [`OutgoingPacket`] if the ack was not received
#[derive(Debug, Message)]
#[rtype(result = "()")]
pub struct ResendPacket {
    /// the packet to resend
    pub packet: Packet,
}
impl Handler<ResendPacket> for InteractionActor {
    type Result = ();
    fn handle(&mut self, mut msg: ResendPacket, ctx: &mut Self::Context) -> Self::Result {
        if self
            .viewer_acks
            .contains(&msg.packet.header.sequence_number)
        {
            msg.packet.header.resent = true;
            ctx.address().do_send(OutgoingPacket { packet: msg.packet });
        }
    }
}

/// Message for sending packets from the core to the server
///
/// Simply a wrapper for the packet struct to send UDP packets to the server
///
/// # Effect
/// - UDP packet sent to the server
/// - [`ResendPacket`] if the packet is marked reliable
#[derive(Debug, Message)]
#[rtype(result = "()")]
pub struct OutgoingPacket {
    /// The outgoing packet
    pub packet: Packet,
}
impl Handler<OutgoingPacket> for InteractionActor {
    type Result = ();
    fn handle(&mut self, mut msg: OutgoingPacket, ctx: &mut Self::Context) -> Self::Result {
        let Some(address) = self.session_context.address.get().cloned() else {
            return;
        };

        if !msg.packet.header.resent {
            msg.packet.header.sequence_number = self.sequence_number as u32;
            self.sequence_number += 1;
        }

        let data = msg.packet.to_bytes().clone();
        let socket_clone = self.socket.as_ref().unwrap().clone();
        let fut = async move {
            if let Err(e) = socket_clone.send_to(&data, &address).await {
                error!("Failed to send data: {}", e);
            }
        };
        ctx.spawn(fut.into_actor(self));

        // if the header is reliable, resend the packet until the viewer_acks contains the key
        if msg.packet.header.reliable {
            self.viewer_acks.insert(msg.packet.header.sequence_number);
            // give one second for the ack to come in.
            // the ResendPacket message check if viewer_acks still contains the sequence
            // number. if it doesn't, that means it's been removed by an ack. If it does,
            // that means it should be resent with the resent flag
            ctx.notify_later(ResendPacket { packet: msg.packet }, Duration::from_secs(1));
        };
    }
}

/// Message for handling region handshakes
///
/// Handles receiving region handshake data, and sending region handshake response packets.
///
/// # Cause
/// - Received a RegionHandshake packet from the UDP socket
///
/// # Effect
/// - Dispatches a [`RegionHandshakeReply`] packet to the server
#[derive(Debug, Message)]
#[rtype(result = "()")]
pub struct HandleRegionHandshake {
    /// The region hanshake data
    pub region_handshake: RegionHandshake,
}
impl Handler<HandleRegionHandshake> for InteractionActor {
    type Result = ();
    fn handle(&mut self, msg: HandleRegionHandshake, ctx: &mut Self::Context) -> Self::Result {
        let (Some(&agent_id), Some(&session_id)) = (
            self.session_context.agent_id.get(),
            self.session_context.session_id.get(),
        ) else {
            return;
        };

        // set some region info
        //session.region_data.water_height = msg.region_handshake.water_height;
        ctx.address().do_send(SendUIMessage {
            ui_message: UIMessage::new_water_update(WaterUpdate {
                height: msg.region_handshake.water_height,
                color: Rgba::from((0.0f32, 94.0, 184.0, 0.5)),
            }),
        });

        ctx.address().do_send(OutgoingPacket {
            packet: Packet::new_region_handshake_reply(RegionHandshakeReply {
                session_id,
                agent_id,
                flags: 0,
            }),
        });
    }
}

/// Message for handling packet acks
///
/// Removes IDs from the outgoing packet ack queue. Received from the server to inform the core
/// that a reliable message sent from the core has successfully been received from the server
///
/// # Cause
/// - Received a PacketAck packet from the UDP socket
#[derive(Debug, Message)]
#[rtype(result = "()")]
pub struct HandlePacketAck {
    /// Ack data
    pub packet_ack: PacketAck,
}
impl Handler<HandlePacketAck> for InteractionActor {
    type Result = ();
    fn handle(&mut self, msg: HandlePacketAck, _ctx: &mut Self::Context) -> Self::Result {
        for id in msg.packet_ack.packet_ids {
            self.viewer_acks.remove(&id);
        }
    }
}

/// Generic Message for handling retries of network dependent assets.
///
/// This sends periodic retry messages for assets that failed download with recoverable errors.
///
/// # Cause
/// -[`crate::objects::DownloadMeshObject`]
/// -[`crate::objects::ParametricPrimObjectUpdate`]
/// -[`crate::objects::SculptObjectUpdate`]
#[derive(Debug, Message, Clone)]
#[rtype(result = "()")]
pub struct RetryMessage<Retriable>
where
    Retriable: Send + actix::Message,
    Retriable::Result: Send,
    SessionActor: Handler<Retriable>,
{
    /// Message to retry
    pub message: Retriable,
    /// Info message to dislpay upon retry
    pub info_message: String,

    /// Flag set when the local network is overloaded.
    /// this should be handled differently than a network error.
    pub network_overloaded: bool,
}
impl<Retriable> Handler<RetryMessage<Retriable>> for SessionActor
where
    Retriable: Send + 'static + actix::Message + RetriableData,
    Retriable::Result: Send,
    SessionActor: Handler<Retriable>,
{
    type Result = ();

    fn handle(
        &mut self,
        mut msg: RetryMessage<Retriable>,
        ctx: &mut Self::Context,
    ) -> Self::Result {
        let retries = msg.message.retry_count();

        let (max_retries, delay) = (DEFAULT_MAX_RETRIES, backoff_ms(retries));

        // if the error is no route to host, that means the network is probably exhausted.
        // wait an extra 60s for the ports to clear, to avoid flooding
        if msg.network_overloaded {
            warn!(
                "Messages are failing because your network may be overloaded. This is likely a bug! Please report it!"
            )
        }

        if retries >= max_retries {
            error!(
                "Dropping message {} after max retries {}",
                type_name::<Retriable>(),
                max_retries
            );
            return;
        }

        msg.message.inc_retry_count();

        info!("{}", msg.info_message);

        warn!(
            "Message retrying. Retry {} #{} in {} ms",
            type_name::<Retriable>(),
            retries + 1,
            delay
        );

        let addr = ctx.address();

        ctx.spawn(
            async move {
                actix::clock::sleep(std::time::Duration::from_millis(delay)).await;

                addr.do_send(msg.message);
            }
            .into_actor(self),
        );
    }
}
fn backoff_ms(retry: u32) -> u64 {
    let base_ms = 10_000;
    let max_ms = 1_000_000;

    let max = (base_ms * 2_u64.saturating_pow(retry)).min(max_ms);

    rand::rng().random_range(50..=max)
}

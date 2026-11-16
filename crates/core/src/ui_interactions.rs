use crate::login::handle_login;
use crate::objects::HandleOrphanedObject;
use crate::{actors::InteractionActor, session::OutgoingPacket};
use actix::AsyncContext;
use actix::{Handler, Message};
use benthic_protocol::messages::ui_protocol::{UIMessage, UIResponse};
use log::error;
use metaverse_messages::packet::packet_protocol::Packet;
use metaverse_messages::udp::agent::agent_update::AgentUpdate;
use metaverse_messages::udp::chat::chat_from_viewer::ChatFromViewer;
use metaverse_messages::udp::core::logout_request::LogoutRequest;
use std::net::UdpSocket;

/// Message for receiving updates from the UI to the core
///
/// # Cause
/// - Received a UIRespones message forom the UI-Core UDP socket
#[derive(Debug, Message)]
#[rtype(result = "()")]
pub struct HandleUIResponse {
    /// UI response data
    pub ui_response: UIResponse,
}
impl Handler<HandleUIResponse> for InteractionActor {
    type Result = ();
    fn handle(&mut self, msg: HandleUIResponse, ctx: &mut Self::Context) -> Self::Result {
        // handle login before session is established
        if let UIResponse::Login(data) = &msg.ui_response {
            let ctx_addr = ctx.address().clone();
            let mailbox_addr = self.mailbox_addr.clone();
            let login_data = data.clone();
            let session_ctx = self.session_context.clone();
            actix::spawn(async move {
                if let Err(e) =
                    handle_login(login_data, &ctx_addr, &mailbox_addr, session_ctx).await
                {
                    error!("Login Error:{:?}", e);
                };
            });
        }
        // other messages require session
        if let (Some(&agent_id), Some(&session_id)) = (
            self.session_context.agent_id.get(),
            self.session_context.session_id.get(),
        ) {
            match msg.ui_response {
                UIResponse::ChatFromViewer(data) => {
                    ctx.address().do_send(OutgoingPacket {
                        packet: Packet::new_chat_from_viewer(ChatFromViewer {
                            session_id,
                            agent_id,
                            channel: data.channel,
                            message: data.message,
                            message_type: data.message_type,
                        }),
                    });
                }
                UIResponse::Logout(_) => {
                    ctx.address().do_send(OutgoingPacket {
                        packet: Packet::new_logout_request(LogoutRequest {
                            session_id,
                            agent_id,
                        }),
                    });
                }
                UIResponse::AgentUpdate(data) => {
                    ctx.address().do_send(OutgoingPacket {
                        packet: Packet::new_agent_update(AgentUpdate {
                            session_id: data.session_id,
                            agent_id: data.session_id,
                            body_rotation: data.body_rotation,
                            head_rotation: data.head_rotation,
                            state: data.state,
                            camera_center: data.camera_center,
                            camera_at_axis: data.camera_at_axis,
                            camera_left_axis: data.camera_left_axis,
                            camera_up_axis: data.camera_up_axis,
                            far: data.far,
                            control_flags: data.control_flags,
                            flags: data.flags,
                        }),
                    });
                }
                UIResponse::OrphanedObject(data) => {
                    self.mailbox_addr.do_send(HandleOrphanedObject(data));
                }
                data => {
                    error!("Unrecognized UIMessage: {:?}", data)
                }
            }
        }
    }
}

/// Message for sending data from the core to the UI
///
/// # Effect
/// - UDP packet sent to UI
#[derive(Debug, Message)]
#[rtype(result = "()")]
pub struct SendUIMessage {
    /// UI response data
    pub ui_message: UIMessage,
}
impl Handler<SendUIMessage> for InteractionActor {
    type Result = ();
    fn handle(&mut self, msg: SendUIMessage, _: &mut Self::Context) -> Self::Result {
        let client_socket = UdpSocket::bind("0.0.0.0:0").unwrap();
        if let Err(e) = client_socket.send_to(&msg.ui_message.to_bytes(), &self.server_to_ui_socket)
        {
            error!("Failed to send UI message:{:?}", e)
        }
    }
}

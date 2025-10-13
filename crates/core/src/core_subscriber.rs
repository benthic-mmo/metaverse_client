use crate::{
    core::{
        environment::EnvironmentCache,
        inventory::{InventoryData, RefreshInventoryEvent},
        session::{handle_login, Mailbox, Session},
    },
    http_handler::login_to_simulator,
};
use log::{error, info, warn};
use metaverse_messages::{
    errors::errors::{CapabilityError, CircuitCodeError, CompleteAgentMovementError},
    http::capabilities::{Capability, CapabilityRequest},
    packet::{
        message::{UIMessage, UIResponse},
        packet::Packet,
    },
    udp::core::{circuit_code::CircuitCode, complete_agent_movement::CompleteAgentMovementData},
    ui::{
        errors::{MailboxSessionError, SessionError},
        login_event::Login,
        login_response::LoginResponse as LoginResponseUiMessage,
    },
};
use std::sync::Mutex;
use std::{collections::HashMap, sync::Arc};
use tokio::net::UdpSocket;

/// This is used to enable the core to listen to messages coming in from the UI.
/// Messages from the UI are sent in bytes as packets, and deserialized in the same way that they
/// would be sent to and from the core
/// all of these packets and their byte representations are defined by the spec here.
/// <https://wiki.secondlife.com/wiki/Category:Messages>
/// Messages are sent to the core using UDS.
///
/// Once this is running, users can send messages like
/// ```rust
/// use metaverse_messages::packet::Packet;
/// use metaverse_messages::login::login::Login;
/// use std::os::net::UdpSocket;
/// use portpicker::pick_unused_port;
///
/// let packet = Packet::new_login_packet(Login {
///            first: "default".to_string(),
///            last: "user".to_string(),
///            passwd: "password".to_string(),
///            start: "home".to_string(),
///            channel: "benthic".to_string(),
///            agree_to_tos: true,
///            read_critical: true,
///            url: "http://127.0.0.1:9000".to_string(),
///        })
///        .to_bytes();
///
/// let client_socket = UdpSocket::bind("0.0.0.0:0").unwrap();
/// let ui_to_core_socket = pick_unused_port().map_or_else(|| "No port found".to_string(), |port| port.to_string());
/// match client_socket.send_to(&packet, format!("127.0.0.1:{}", ui_to_core_socket)) {
///     Ok(_) => println!("Login sent from UI"),
///     Err(e) => println!("Error sending login from UI {:?}", e),
/// };
///
/// ```
pub async fn listen_for_ui_messages(ui_to_core_socket: String, mailbox_addr: actix::Addr<Mailbox>) {
    let socket = UdpSocket::bind(ui_to_core_socket)
        .await
        .expect("Failed to bind to UDP socket");
    loop {
        let mut buf = [0u8; 1500];
        match socket.recv_from(&mut buf).await {
            Ok((n, _)) => {
                let event = match UIResponse::from_bytes(&buf[..n]) {
                    Ok(event) => event,
                    Err(e) => {
                        warn!("Failed to receive event {:?},", e);
                        continue;
                    }
                };
                if let UIResponse::Login(login) = event {
                    match handle_login(login.clone(), &mailbox_addr).await {
                        Ok(_) => info!("Successfully logged in"),
                        Err(e) => {
                            // send the error to the UI to handle
                            warn!("Error logging in: {:?}", e);
                            mailbox_addr.do_send(UIMessage::new_session_error(e));
                        }
                    };
                } else {
                    // send the event to the core to handle
                    mailbox_addr.do_send(event);
                }
            }
            Err(e) => {
                warn!("Core failed to read buffer sent from UI {}", e)
            }
        }
    }
}

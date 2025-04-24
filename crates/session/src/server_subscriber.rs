use crate::mailbox::{Mailbox, Session, UiMessage};
use log::{info, warn};
use metaverse_messages::circuit_code::CircuitCodeData;
use metaverse_messages::complete_agent_movement::CompleteAgentMovementData;
use metaverse_messages::errors::{
    CircuitCodeError, CompleteAgentMovementError, MailboxError, SessionError,
};
use metaverse_messages::login_system::login::{login, Login};
use metaverse_messages::login_system::login_response::LoginResponse;
use metaverse_messages::login_system::simulator_login_protocol::SimulatorLoginProtocol;
use metaverse_messages::packet::Packet;
use metaverse_messages::packet_types::PacketType;
use metaverse_messages::ui_events::UiEventTypes;
use tokio::net::UdpSocket;

/// This is used for the server to listen to messages coming in from the UI.
/// Messages from the UI are sent in bytes as packets, and deserialized in the same way that they
/// would be sent to and from the server.
/// all of these packets and their byte representations are defined by the spec here.
/// https://wiki.secondlife.com/wiki/Category:Messages
/// Messages are sent to the server using UDS.
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
/// let ui_to_server_socket = pick_unused_port().map_or_else(|| "No port found".to_string(), |port| port.to_string());
/// match client_socket.send_to(&packet, format!("127.0.0.1:{}", ui_to_server_socket)) {
///     Ok(_) => println!("Login sent from UI"),
///     Err(e) => println!("Error sending login from UI {:?}", e),
/// };
///
/// ```
///
///
pub async fn listen_for_ui_messages(
    ui_to_server_socket: String,
    mailbox_addr: actix::Addr<Mailbox>,
) {
    let socket = UdpSocket::bind(ui_to_server_socket)
        .await
        .expect("Failed to bind to UDP socket");
    loop {
        let mut buf = [0u8; 1500];
        match socket.recv_from(&mut buf).await {
            Ok((n, _)) => {
                let packet = match Packet::from_bytes(&buf[..n]) {
                    Ok(packet) => packet,
                    Err(e) => {
                        warn!("Server failed to receive event from UI: {:?}", e);
                        continue;
                    }
                };
                if let PacketType::Login(login) = packet.body {
                    match handle_login((*login).clone(), &mailbox_addr).await {
                        Ok(_) => info!("Successfully logged in"),
                        Err(e) => {
                            // send the error to the UI to handle
                            warn!("Error logging in: {:?}", e);
                            mailbox_addr.do_send(UiMessage::new(UiEventTypes::Error, e.to_bytes()));
                        }
                    };
                } else {
                    mailbox_addr.do_send(packet);
                }
            }
            Err(e) => {
                warn!("Server failed to read buffer sent from UI {}", e)
            }
        }
    }
}

async fn login_with_creds(login_data: Login) -> Result<LoginResponse, SessionError> {
    let url = login_data.url.clone();
    match login(SimulatorLoginProtocol::new(login_data), url).await {
        Ok(login_result) => Ok(login_result),
        Err(e) => Err(SessionError::new_login_error(e)),
    }
}

async fn handle_login(
    login_data: Login,
    mailbox_addr: &actix::Addr<Mailbox>,
) -> Result<(), SessionError> {
    let login_response = match login_with_creds(login_data).await {
        Ok(response) => {
            let serialized = serde_json::to_string(&response).unwrap();
            if let Err(e) = mailbox_addr
                .send(UiMessage::new(
                    UiEventTypes::LoginResponseEvent,
                    serialized.to_string().into_bytes(),
                ))
                .await
            {
                warn!("Failed to send LoginResponse to Mailbox {:?}", e)
            };
            response
        }
        // returns the session error created by the login_with_creds function
        Err(error) => return Err(error),
    };

    if let Err(e) = mailbox_addr
        .send(Session {
            server_socket: login_response.sim_port.unwrap(),
            url: login_response.sim_ip.unwrap(),
            agent_id: login_response.agent_id.unwrap(),
            session_id: login_response.session_id.unwrap(),
            socket: None,
        })
        .await
    {
        return Err(SessionError::Mailbox(MailboxError::new(format!("{}", e))));
    };

    if let Err(e) = mailbox_addr
        .send(Packet::new_circuit_code(CircuitCodeData {
            code: login_response.circuit_code,
            session_id: login_response.session_id.unwrap(),
            id: login_response.agent_id.unwrap(),
        }))
        .await
    {
        return Err(SessionError::CircuitCode(CircuitCodeError::new(format!(
            "{}",
            e
        ))));
    };

    if let Err(e) = mailbox_addr
        .send(Packet::new_complete_agent_movement(
            CompleteAgentMovementData {
                circuit_code: login_response.circuit_code,
                session_id: login_response.session_id.unwrap(),
                agent_id: login_response.agent_id.unwrap(),
            },
        ))
        .await
    {
        return Err(SessionError::CompleteAgentMovement(
            CompleteAgentMovementError::new(format!("{}", e)),
        ));
    };

    Ok(())
}

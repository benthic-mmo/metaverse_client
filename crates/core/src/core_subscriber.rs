use crate::core::{
    environment::EnvironmentCache,
    inventory::{InventoryData, RefreshInventoryEvent},
    session::{Mailbox, Session, UiMessage},
};
use log::{info, warn};
use metaverse_messages::{
    agent::agent_wearables_request::AgentWearablesRequest,
    capabilities::capabilities::{Capability, CapabilityRequest},
    errors::errors::{CapabilityError, CircuitCodeError, CompleteAgentMovementError},
    login::{
        circuit_code::CircuitCodeData,
        complete_agent_movement::CompleteAgentMovementData,
        login_response::LoginResponse,
        login_xmlrpc::{Login, send_login_xmlrpc},
        simulator_login_protocol::SimulatorLoginProtocol,
    },
    packet::{packet::Packet, packet_types::PacketType},
    ui::{
        errors::{MailboxSessionError, SessionError},
        ui_events::UiEventTypes,
    },
    utils::skeleton::Skeleton,
};
use std::sync::Mutex;
use std::{collections::HashMap, sync::Arc};
use tokio::net::UdpSocket;

/// This is used for the core to listen to messages coming in from the UI.
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
///
///
pub async fn listen_for_ui_messages(ui_to_core_socket: String, mailbox_addr: actix::Addr<Mailbox>) {
    let socket = UdpSocket::bind(ui_to_core_socket)
        .await
        .expect("Failed to bind to UDP socket");
    loop {
        let mut buf = [0u8; 1500];
        match socket.recv_from(&mut buf).await {
            Ok((n, _)) => {
                let packet = match Packet::from_bytes(&buf[..n]) {
                    Ok(packet) => packet,
                    Err(e) => {
                        warn!("Core failed to receive event from UI: {:?}", e);
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
                warn!("Core failed to read buffer sent from UI {}", e)
            }
        }
    }
}

async fn login_with_creds(login_data: Login) -> Result<LoginResponse, SessionError> {
    let url = login_data.url.clone();
    match send_login_xmlrpc(SimulatorLoginProtocol::new(login_data), url).await {
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
            agent_id: login_response.agent_id,
            session_id: login_response.session_id,
            seed_capability_url: login_response.seed_capability.unwrap(),
            capability_urls: HashMap::new(),

            #[cfg(feature = "environment")]
            environment_cache: EnvironmentCache {
                patch_queue: HashMap::new(),
                patch_cache: HashMap::new(),
            },

            #[cfg(feature = "inventory")]
            inventory_data: InventoryData {
                inventory_root: login_response.inventory_root,
                inventory_lib_root: login_response.inventory_lib_root,
                inventory_lib_owner: login_response.inventory_lib_owner,
                inventory_tree: None,
            },
            #[cfg(feature = "agent")]
            agent_list: Arc::new(Mutex::new(HashMap::new())),
            socket: None,
        })
        .await
    {
        return Err(SessionError::MailboxSession(MailboxSessionError::new(
            format!("{}", e),
        )));
    };

    if let Err(e) = mailbox_addr
        .send(Packet::new_circuit_code(CircuitCodeData {
            code: login_response.circuit_code,
            session_id: login_response.session_id,
            id: login_response.agent_id,
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
                session_id: login_response.session_id,
                agent_id: login_response.agent_id,
            },
        ))
        .await
    {
        return Err(SessionError::CompleteAgentMovement(
            CompleteAgentMovementError::new(format!("{}", e)),
        ));
    };
    if let Err(e) = mailbox_addr
        .send(Packet::new_agent_wearables_request(AgentWearablesRequest {
            agent_id: login_response.agent_id,
            session_id: login_response.session_id,
        }))
        .await
    {
        return Err(SessionError::CompleteAgentMovement(
            CompleteAgentMovementError::new(format!("{}", e)),
        ));
    };
    if let Err(e) = mailbox_addr
        .send(CapabilityRequest::new_capability_request(vec![
            #[cfg(any(feature = "agent", feature = "environment"))]
            Capability::ViewerAsset,
            #[cfg(feature = "inventory")]
            Capability::FetchLibDescendents2,
            #[cfg(feature = "inventory")]
            Capability::FetchInventoryDescendents2,
        ]))
        .await
    {
        return Err(SessionError::Capability(CapabilityError::new(format!(
            "{}",
            e
        ))));
    }

    #[cfg(feature = "inventory")]
    if let Err(e) = mailbox_addr
        .send(RefreshInventoryEvent {
            agent_id: login_response.agent_id,
        })
        .await
    {
        return Err(SessionError::Capability(CapabilityError::new(format!(
            "failed to retrieve inventory {}",
            e
        ))));
    }

    Ok(())
}

use log::{error, info};
use metaverse_login::login::login;
use metaverse_login::models::errors::{LoginError, Reason};
use metaverse_login::models::login_response::LoginResponse;
use metaverse_login::models::simulator_login_protocol::SimulatorLoginProtocol;
use metaverse_messages::models::circuit_code::CircuitCodeData;
use metaverse_messages::models::complete_agent_movement::CompleteAgentMovementData;
use metaverse_messages::models::login::Login;
use metaverse_messages::models::packet::Packet;
use std::path::PathBuf;
use tokio::net::UnixDatagram;

use crate::errors::{CircuitCodeError, CompleteAgentMovementError, SendFailReason, SessionError};
use crate::mailbox::{Mailbox, Session, TriggerSend};

pub async fn listen(socket_path: PathBuf, mailbox_addr: actix::Addr<Mailbox>) {
    let socket = UnixDatagram::bind(socket_path.clone()).unwrap();
    info!(
        "Incoming message handler listening on UDS: {:?}",
        socket_path
    );

    loop {
        let mut buf = [0u8; 1024];
        match socket.recv_from(&mut buf).await {
            Ok((n, _)) => {
                info!("Incoming Message Handler receiving data {}", n);
                let packet = match Packet::from_bytes(&buf[..n]) {
                    Ok(packet) => packet,
                    Err(e) => {
                        error!("Incoming Message Handler Failed to parse packet {:?}", e);
                        continue;
                    }
                };
                if let Some(login) = packet.body.as_any().downcast_ref::<Login>() {
                    match handle_login((*login).clone(), &mailbox_addr).await {
                        Ok(_) => info!("Successfully logged in"),
                        Err(e) => error!("Failed to log in {:?}", e),
                    };
                } else {
                    mailbox_addr.do_send(packet);
                }
            }
            Err(e) => {
                error!("Failed to read buffer {}", e)
            }
        }
    }
}

async fn login_with_creds(login_data: Login) -> Result<LoginResponse, SessionError> {
    let login_result = tokio::task::spawn_blocking(|| {
        let url = login_data.url.clone();
        login(SimulatorLoginProtocol::new(login_data), url)
    });

    match login_result.await {
        Ok(Ok(response)) => Ok(response),
        Ok(Err(e)) => Err(SessionError::new_login_error(e)),
        Err(e) => Err(SessionError::new_login_error(LoginError::new(
            Reason::Unknown,
            &format!("join error: {}", e),
        ))),
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
                .send(TriggerSend {
                    message_type: "LoginResponse".to_string(),
                    total_packet_number: 0,
                    sequence_number: 0,
                    encoding: "json".to_string(),
                    message: serialized.to_string(),
                })
                .await
            {
                error!("Failed to send LoginResponse to Mailbox {:?}", e)
            };
            response
        }
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
        let error = SessionError::CircuitCode(CircuitCodeError::new(
            SendFailReason::Timeout,
            format!("THIS IS A PLACEHOLDER ERROR IT'S NOT RIGHT PLEASE FIX{}", e),
        ));
        return Err(error);
    };

    if let Err(e) = mailbox_addr
        .send(Packet::new_circuit_code(CircuitCodeData {
            code: login_response.circuit_code,
            session_id: login_response.session_id.unwrap(),
            id: login_response.agent_id.unwrap(),
        }))
        .await
    {
        let error = SessionError::CircuitCode(CircuitCodeError::new(
            SendFailReason::Timeout,
            format!("{}", e),
        ));
        return Err(error);
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
        let error = SessionError::CompleteAgentMovement(CompleteAgentMovementError::new(
            SendFailReason::Timeout,
            format!("{}", e),
        ));
        return Err(error);
    };
    Ok(())
}

use actix::{Actor, Addr};
use log::info;
use metaverse_login::login::{self};
use metaverse_login::models::errors::{LoginError, Reason};
use metaverse_login::models::simulator_login_protocol::{Login, SimulatorLoginProtocol};
use metaverse_messages::models::circuit_code::CircuitCodeData;
use metaverse_messages::models::client_update_data::send_message_to_client;
use metaverse_messages::models::client_update_data::{ClientUpdateData, LoginProgress};
use metaverse_messages::models::complete_agent_movement::CompleteAgentMovementData;
use metaverse_messages::models::packet::Packet;
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::Mutex;
use uuid::Uuid;

use tokio::time::Duration;

use crate::models::errors::{
    CircuitCodeError, CompleteAgentMovementError, SendFailReason, SessionError,
};
use crate::models::mailbox::{AllowAcks, Mailbox};

pub struct Session {
    pub mailbox: Addr<Mailbox>,
    pub update_stream: Arc<Mutex<Vec<ClientUpdateData>>>,
    pub agent_id: Uuid,
    pub session_id: Uuid,
}

impl Session {
    pub async fn new(
        login_data: Login,
        login_url: String,
        update_stream: Arc<Mutex<Vec<ClientUpdateData>>>,
    ) -> Result<Self, SessionError> {
        let packet_sequence_number = Arc::new(Mutex::new(0u32));

        send_message_to_client(
            update_stream.clone(),
            LoginProgress {
                message: "Sending login xml".to_string(),
                percent: 5,
            }
            .into(),
        )
        .await;

        let login_url_clone = login_url.clone();
        let login_result = tokio::task::spawn_blocking(|| {
            login::login(SimulatorLoginProtocol::new(login_data), login_url_clone)
        });

        let update_stream_clone = update_stream.clone();
        let login_response = match login_result.await {
            Ok(Ok(response)) => {
                send_message_to_client(
                    update_stream_clone,
                    LoginProgress {
                        message: "Login xml sent successfully".to_string(),
                        percent: 10,
                    }
                    .into(),
                )
                .await;
                response
            }
            Ok(Err(e)) => {
                let error = SessionError::new_login_error(e);
                send_message_to_client(update_stream, error.as_boxed_error().into()).await;
                return Err(error);
            }
            Err(e) => {
                let error = SessionError::new_login_error(LoginError::new(
                    Reason::Unknown,
                    &format!("join error: {}", e),
                ));
                send_message_to_client(update_stream, error.as_boxed_error().into()).await;
                return Err(error);
            }
        };

        let ack_queue = Arc::new(Mutex::new(HashMap::new()));
        let command_queue = Arc::new(Mutex::new(HashMap::new()));
        let data_queue = Arc::new(Mutex::new(HashMap::new()));
        let error_queue = Arc::new(Mutex::new(HashMap::new()));
        let request_queue = Arc::new(Mutex::new(HashMap::new()));
        let event_queue = Arc::new(Mutex::new(HashMap::new()));
        let command_queue_clone = command_queue.clone();
        let data_queue_clone = data_queue.clone();
        let error_queue_clone = error_queue.clone();
        let request_queue_clone = request_queue.clone();
        let event_queue_clone = event_queue.clone();
        let ack_queue_clone = ack_queue.clone();
        let update_stream_clone = update_stream.clone();
        let packet_sequence_number_clone = packet_sequence_number.clone();

        send_message_to_client(
            update_stream.clone(),
            LoginProgress {
                message: "Starting packet mailbox".to_string(),
                percent: 25,
            }
            .into(),
        )
        .await;
        let mailbox = Mailbox {
            socket: None,
            url: login_response.sim_ip.unwrap(),
            server_socket: login_response.sim_port.unwrap(),
            client_socket: 41518, //TODO: Make this configurable
            ack_queue: ack_queue_clone,
            command_queue: command_queue_clone,
            data_queue: data_queue_clone,
            error_queue: error_queue_clone,
            request_queue: request_queue_clone,
            event_queue: event_queue_clone,
            update_stream: update_stream_clone,
            packet_sequence_number: packet_sequence_number_clone,
        }
        .start();

        send_message_to_client(
            update_stream.clone(),
            LoginProgress {
                message: "Packet mailbox started".to_string(),
                percent: 40,
            }
            .into(),
        )
        .await;
        send_message_to_client(
            update_stream.clone(),
            LoginProgress {
                message: "Sending use circuit code packet".to_string(),
                percent: 55,
            }
            .into(),
        )
        .await;

            match mailbox
                .send_with_ack(
                    Packet::new_circuit_code(
                        CircuitCodeData {
                            code: login_response.circuit_code,
                            session_id: login_response.session_id.unwrap(),
                            id: login_response.agent_id.unwrap(),
                        },
                    ),
                    Duration::from_secs(1),
                    10,
                )
                .await
            {
                Ok(_) => {
                    send_message_to_client(
                        update_stream.clone(),
                        LoginProgress {
                            message: "Circuit code sent and server ack received".to_string(),
                            percent: 60,
                        }
                        .into(),
                    )
                    .await;
                    info!("circuit code sent and ack received");
                }
                Err(e) => {
                    let error = SessionError::CircuitCode(CircuitCodeError::new(
                        SendFailReason::Timeout,
                        format!("{}", e),
                    ));
                    send_message_to_client(update_stream, error.as_boxed_error().into()).await;
                    return Err(error);
                }
            };

        send_message_to_client(
            update_stream.clone(),
            LoginProgress {
                message: "Sending complete agent movement data packet".to_string(),
                percent: 75,
            }
            .into(),
        )
        .await;

            match mailbox
                .send(Packet::new_complete_agent_movement(
                    CompleteAgentMovementData {
                        circuit_code: login_response.circuit_code,
                        session_id: login_response.session_id.unwrap(),
                        agent_id: login_response.agent_id.unwrap(),
                    },
                ))
                .await
            {
                Ok(_) => {
                    send_message_to_client(
                        update_stream.clone(),
                        LoginProgress {
                            message: "Complete agent movement sent".to_string(),
                            percent: 90,
                        }
                        .into(),
                    )
                    .await;
                    info!("Complete agent movement sent");
                }
                Err(e) => {
                    let error = SessionError::CompleteAgentMovement(
                        CompleteAgentMovementError::new(SendFailReason::Timeout, format!("{}", e)),
                    );
                    send_message_to_client(update_stream, error.as_boxed_error().into()).await;
                    return Err(error);
                }
            };
        send_message_to_client(
            update_stream.clone(),
            LoginProgress {
                message: "Login complete!".to_string(),
                percent: 100,
            }
            .into(),
        )
        .await;

        Ok(Session {
            mailbox,
            update_stream,
            agent_id: login_response.agent_id.unwrap(),
            session_id: login_response.session_id.unwrap(),
        })
    }
}

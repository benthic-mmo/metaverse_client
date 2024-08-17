use actix::{Actor, Addr};
use log::info;
use metaverse_login::login::{self};
use metaverse_login::models::errors::{LoginError, Reason};
use metaverse_login::models::simulator_login_protocol::{Login, SimulatorLoginProtocol};
use metaverse_messages::models::circuit_code::CircuitCodeData;
use metaverse_messages::models::client_update_data::{
    ClientUpdateContent, ClientUpdateData, DataContent,
};
use metaverse_messages::models::complete_agent_movement::CompleteAgentMovementData;
use metaverse_messages::models::packet::Packet;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

use tokio::time::Duration;

use crate::models::errors::{
    CircuitCodeError, CompleteAgentMovementError, SendFailReason, SessionError,
};
use crate::models::mailbox::{AllowAcks, Mailbox};

pub struct Session {
    pub mailbox: Addr<Mailbox>,
    pub update_stream: Arc<Mutex<Vec<ClientUpdateData>>>,
}

impl Session {
    pub async fn new(
        login_data: Login,
        login_url: String,
        update_stream: Arc<Mutex<Vec<ClientUpdateData>>>,
    ) -> Result<Self, SessionError> {
        let login_url_clone = login_url.clone();
        let login_result = tokio::task::spawn_blocking(|| {
            login::login(SimulatorLoginProtocol::new(login_data), login_url_clone)
        });

        let login_response = match login_result.await {
            Ok(Ok(response)) => response,
            Ok(Err(e)) => return Err(SessionError::new_login_error(e)),
            Err(e) => {
                return Err(SessionError::new_login_error(LoginError::new(
                    Reason::Unknown,
                    &format!("join error: {}", e),
                )))
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
        }
        .start();
        
        match mailbox
            .send_with_ack(
                Packet::new_circuit_code(CircuitCodeData {
                    code: login_response.circuit_code,
                    session_id: login_response.session_id.unwrap(),
                    id: login_response.agent_id.unwrap(),
                }),
                Duration::from_secs(1),
                10,
            )
            .await
        {
            Ok(_) => info!("circuit code sent and ack received"),
            Err(e) => {
                return Err(SessionError::CircuitCode(CircuitCodeError::new(
                    SendFailReason::Timeout,
                    format!("{}", e),
                )))
            }
        };
        match mailbox
            .send(
                Packet::new_complete_agent_movement(CompleteAgentMovementData {
                    circuit_code: login_response.circuit_code,
                    session_id: login_response.session_id.unwrap(),
                    agent_id: login_response.agent_id.unwrap(),
                }),
            )
            .await
        {
            Ok(_) => info!("complete agent movement sent"),
            Err(e) => {
                return Err(SessionError::CompleteAgentMovement(
                    CompleteAgentMovementError::new(SendFailReason::Timeout, format!("{}", e)),
                ))
            }
        };
        Ok(Session {
            mailbox,
            update_stream,
        })
    }
}

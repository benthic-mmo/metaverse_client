use actix::Actor;
use log::info;
use metaverse_login::login::{self};
use metaverse_login::models::login_response::LoginResult;
use metaverse_login::models::simulator_login_protocol::Login;
use metaverse_messages::models::circuit_code::CircuitCodeData;
use metaverse_messages::models::complete_agent_movement::CompleteAgentMovementData;
use metaverse_messages::models::packet::Packet;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

use std::error::Error;
use tokio::time::Duration;

use crate::models::mailbox::{AllowAcks, Mailbox};

pub async fn new_session(login_data: Login, login_url: String) -> Result<(), Box<dyn Error>> {
    let login_url_clone = login_url.clone();

    let login_result = tokio::task::spawn_blocking(|| {
        login::login(login::build_login(login_data), login_url_clone).unwrap()
    });
    let login_response = match login_result.await {
        Ok(LoginResult::Success(response)) => response,
        Ok(LoginResult::Failure(failure)) => {
            return Err(format!("Login failed with: {}", failure.message).into());
        }
        Err(e) => return Err(format!("Login failed with unknown error, {}", e).into()),
    };
    let ack_queue = Arc::new(Mutex::new(HashMap::new()));
    let command_queue = Arc::new(Mutex::new(HashMap::new()));
    let data_queue = Arc::new(Mutex::new(HashMap::new()));
    let error_queue = Arc::new(Mutex::new(HashMap::new()));
    let request_queue = Arc::new(Mutex::new(HashMap::new()));
    let event_queue = Arc::new(Mutex::new(HashMap::new()));
    let clone = ack_queue.clone();
    let mailbox = Mailbox {
        socket: None,
        url: login_response.sim_ip.unwrap(),
        server_socket: login_response.sim_port.unwrap(),
        client_socket: 41518, //TODO: Make this configurable
        ack_queue: clone,
        command_queue,
        data_queue,
        error_queue,
        request_queue,
        event_queue,
    }
    .start();

    // send circuit code and await its ack
    match mailbox
        .send_with_ack(
            Packet::new_circuit_code(CircuitCodeData {
                code: login_response.circuit_code,
                session_id: login_response.session_id.unwrap(),
                id: login_response.agent_id.unwrap(),
            }),
            Duration::from_millis(500),
            3,
        )
        .await
    {
        Ok(_) => info!("circuit code sent and ack received"),
        Err(e) => {
            return Err(format!("Failed to create connection {}", e).into())
        },
    };

    match mailbox
        .send_with_ack(
            Packet::new_complete_agent_movement(CompleteAgentMovementData {
                circuit_code: login_response.circuit_code,
                session_id: login_response.session_id.unwrap(),
                agent_id: login_response.agent_id.unwrap(),
            }),
            Duration::from_millis(500),
            3,
        )
        .await
    {
        Ok(_) => info!("complete agent movement sent and ack received"),
        Err(e) => {
            return Err(format!("Failed to complete agent movement: {}", e).into())
        },
    };



    Ok(())
}

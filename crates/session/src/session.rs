use actix::Actor;
use metaverse_login::login::{self};
use metaverse_login::models::login_response::LoginResult;
use metaverse_login::models::simulator_login_protocol::Login;
use metaverse_messages::models::circuit_code::CircuitCodeData;
use metaverse_messages::models::packet::Packet;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::oneshot;
use tokio::sync::Mutex;

use std::error::Error;
use tokio::time::{sleep, Duration};

use crate::models::mailbox::Mailbox;

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

    let mut attempts = 0;
    let mut received_ack = false;

    sleep(Duration::from_secs(2)).await;

    while attempts < 3 && !received_ack {
        let (tx, rx) = oneshot::channel();

        {
            let mut queue = ack_queue.lock().await;
            queue.insert(1, tx);
        }

        mailbox.do_send(Packet::new_circuit_code(CircuitCodeData {
            code: login_response.circuit_code,
            session_id: login_response.session_id.unwrap(),
            id: login_response.agent_id.unwrap(),
        }));

        tokio::select! {
            _ = rx => {
                received_ack = true; // Received acknowledgment
            },
            _ = sleep(Duration::from_millis(500)) => {
                attempts += 1;
                if !received_ack {
                    {
                        let mut queue = ack_queue.lock().await;
                        queue.remove(&1);
                    }
                }
            }
        }
    }
    Ok(())
}

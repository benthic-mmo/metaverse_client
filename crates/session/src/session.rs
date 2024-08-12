use actix::Actor;
use metaverse_login::login::{self};
use metaverse_login::models::login_response::LoginResult;
use metaverse_login::models::simulator_login_protocol::Login;
use metaverse_messages::models::packet::Packet;
use metaverse_messages::models::use_circuit_code::CircuitCodeData;

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

    let mailbox = Mailbox {
        socket: None,
        url: login_response.sim_ip.unwrap(),
        server_socket: login_response.sim_port.unwrap(),
        client_socket: 41518, //TODO: Make this configurable
    }
    .start();

    let mut attempts = 0;

    let packet = Packet::new_circuit_code(CircuitCodeData {
        code: login_response.circuit_code,
        session_id: login_response.session_id.unwrap(),
        id: login_response.agent_id.unwrap(),
    });

    sleep(Duration::from_secs(2)).await;

    while attempts < 3 {
        mailbox.do_send(crate::models::mailbox::Packet {
            data: packet.to_bytes(),
        });
        attempts += 1;
        sleep(Duration::from_millis(500)).await;
    }
    Ok(())
}

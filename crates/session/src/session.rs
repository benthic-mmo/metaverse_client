use actix::Actor;
use metaverse_login::login::{self};
use metaverse_login::models::login_response::LoginResult;
use metaverse_login::models::simulator_login_protocol::Login;
use metaverse_messages::models::use_circuit_code::*;

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
        client_socket: 41518,
    }.start();

    let mut attempts = 0;

    let packet = UseCircuitCodePacket::new(
        login_response.circuit_code,
        login_response.session_id.unwrap(),
        login_response.agent_id.unwrap(),
    );
    let bytes = packet.to_bytes();
    let new_packet = UseCircuitCodePacket::from_bytes(&bytes).await?;
    println!("{:?}", new_packet);

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

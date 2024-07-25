use metaverse_login::login::{self};
use metaverse_login::models::login_response::LoginResult;
use metaverse_login::models::simulator_login_protocol::Login;
use metaverse_messages::models::use_circuit_code::*;

use std::{error::Error, io};
use tokio::net::UdpSocket;


pub async fn new_session(login_data: Login, login_url: String) -> Result<(), Box<dyn Error>>{
    let login_result = tokio::task::spawn_blocking(|| {
        login::login(login::build_login(login_data),login_url).unwrap()
    });
   
    let login_response = match login_result.await {
        Ok(LoginResult::Success(response)) => {
            response 
        }
        Ok(LoginResult::Failure(failure)) => {
            return Err(format!("Login failed with: {}", failure.message).into());
        }
        Err(e) => return Err(format!("Login failed with unknown error").into()),
    };
    let mut session_addr = login_response.sim_ip.unwrap().clone();
    session_addr.push(':'); 
    session_addr.push_str(&login_response.sim_port.unwrap().to_string()); 
    let sock = UdpSocket::bind(login_url).await?; 
    let packet = create_use_circuit_code_packet(create_use_circuit_code(
        login_response.circuit_code,
        login_response.session_id.unwrap(),
        login_response.agent_id.unwrap()
    )).unwrap(); 

    sock.send(&packet).await?; 
    Ok(())
}

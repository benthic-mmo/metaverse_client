use metaverse_login::login::{self};
use metaverse_login::models::login_response::LoginResult;
use metaverse_login::models::simulator_login_protocol::Login;
use metaverse_messages::models::use_circuit_code::*;
use log::{info, error};

use std::error::Error;
use tokio::net::UdpSocket;


pub async fn new_session(login_data: Login, login_url: String) -> Result<(), Box<dyn Error>>{
    let login_url_clone = login_url.clone();
    
    let login_result = tokio::task::spawn_blocking(|| {
        login::login(login::build_login(login_data),login_url_clone).unwrap()
    });
    let login_response = match login_result.await {
        Ok(LoginResult::Success(response)) => {
            response
        }
        Ok(LoginResult::Failure(failure)) => {
            return Err(format!("Login failed with: {}", failure.message).into());
        }
        Err(e) => return Err(format!("Login failed with unknown error, {}", e).into()),
    };
    let mut session_addr = login_response.sim_ip.unwrap().clone();
    session_addr.push(':'); 
    session_addr.push_str(&login_response.sim_port.unwrap().to_string()); 
   
    let local_socket = "127.0.0.1:41518";
    let sock = match UdpSocket::bind(local_socket).await {
        Ok(sock) => {
            println!("Successfully bound to {}", session_addr);
            sock 
            // Use sock here
        }
        Err(e) => {
            error!("Failed to bind to {}: {}", session_addr, e);
            
            return Err("Failed to create sock".into())
        }
    };
    info!("socket bound");
    let packet = create_use_circuit_code_packet(create_use_circuit_code(
        login_response.circuit_code,
        login_response.session_id.unwrap(),
        login_response.agent_id.unwrap()
    )).unwrap();

    match String::from_utf8(packet.clone()) {
        Ok(string) => info!("Converted string: {}", string),
        Err(e) => info!("Failed to convert bytes to string: {}", e),
    }
    match sock.send_to(&packet, session_addr).await {
        Ok(_) => {
            info!("Sent packet")
        }, 
        Err(e) => {
            error!("failed to send packet: {}", e)
        }
    };
    Ok(())
}

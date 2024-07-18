use crate::models::errors::{LoginError, Reason};
use crate::models::session_data::Session;
use metaverse_messages::models::use_circuit_code::*;

use std::io;
use tokio::net::UdpSocket;

pub fn new_session(login_response: xmlrpc::Value) -> Result<Session, LoginError> {
    match login_response["login"].as_str().unwrap() {
        // if login is true then login succeeded
        "true" => Ok(login_response.into()),
        // if login is false then login failed
        "false" => match login_response.get("reason") {
            // if no reason is sent, something strange happened
            None => Err(LoginError {
                reason: Reason::Unknown,
                message: format!("{:?}", login_response),
            }),
            Some(x) => match x.as_str().unwrap() {
                "key" => Err(LoginError {
                    reason: Reason::Key,
                    message: login_response["message"].as_str().unwrap().to_string(),
                }),
                "presence" => Err(LoginError {
                    reason: Reason::Presence,
                    message: login_response["message"].as_str().unwrap().to_string(),
                }),
                _ => Err(LoginError {
                    reason: Reason::Unknown,
                    message: login_response["message"].as_str().unwrap().to_string(),
                }),
            },
        },
        &_ => Err(LoginError {
            reason: Reason::Unknown,
            message: "".to_string(),
        }),
    }
}

pub async fn connect(session: Session) -> io::Result<()> {
    let mut session_addr = session.sim_ip.unwrap().clone();
    session_addr.push(':');
    session_addr.push_str(&session.sim_port.unwrap().to_string());
    let sock = UdpSocket::bind("127.0.0.1:0").await?;
    sock.connect(session_addr.clone()).await?;

    let packet = create_use_circuit_code_packet(create_use_circuit_code(
        session.circuit_code.unwrap(),
        session.session_id.unwrap(),
        session.agent_id.unwrap(),
    ))
    .unwrap();
    sock.send(&packet).await?;
    Ok(())
}

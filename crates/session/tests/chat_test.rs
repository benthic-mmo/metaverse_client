use actix::System;
use crossbeam_channel::{Sender, unbounded};
use log::{LevelFilter, error, info};
use metaverse_messages::login_system::login_response::LoginResponse;
use metaverse_messages::packet_types::PacketType;
use tempfile::NamedTempFile;

use metaverse_messages::chat_from_viewer::{ChatFromViewer, ClientChatType};
use metaverse_messages::login_system::login::Login;
use metaverse_messages::packet::Packet;
use metaverse_session::client_subscriber::listen_for_server_events;
use metaverse_session::initialize::initialize;

use std::os::unix::net::UnixDatagram;
use std::path::PathBuf;
use std::thread::sleep;
use std::time::Duration;

#[test]
fn test_chat() {
    let incoming_socket_path = NamedTempFile::new()
        .expect("Failed to create temp file")
        .path()
        .to_path_buf();

    let (sender, receiver) = unbounded();
    init_tests(incoming_socket_path.clone(), sender);

    // wait for the mailbox to be ready. This can be done in a better way.
    sleep(Duration::from_secs(2));

    // login so you can send a chat message
    let message = Packet::new_login_packet(Login {
        first: "default".to_string(),
        last: "user".to_string(),
        passwd: "password".to_string(),
        start: "home".to_string(),
        channel: "benthic".to_string(),
        agree_to_tos: true,
        read_critical: true,
        url: build_test_url("http://127.0.0.1", 9000).to_string(),
    })
    .to_bytes();
    let client_socket = UnixDatagram::unbound().unwrap();
    match client_socket.send_to(&message, &incoming_socket_path) {
        Ok(_) => println!("message sent to mailbox"),
        Err(e) => println!("error sending to mailbox {:?}", e),
    };
    sleep(Duration::from_secs(3));

    // get the login response from the server
    let mut outer_login_response: Option<LoginResponse> = None;
    while let Ok(event) = receiver.try_recv() {
        if let PacketType::LoginResponse(login_response) = event {
            // if the test receives a LoginResponse, then it passes
            outer_login_response = Some(*login_response);
        }
    }

    // send a chat message
    if let Some(response) = outer_login_response {
        let message = Packet::new_chat_from_viewer(ChatFromViewer {
            agent_id: response.agent_id.unwrap(),
            session_id: response.session_id.unwrap(),
            channel: 0,
            message_type: ClientChatType::Normal,
            message: "hello".to_string(),
        })
        .to_bytes();
        let client_socket = UnixDatagram::unbound().unwrap();
        match client_socket.send_to(&message, &incoming_socket_path) {
            Ok(_) => println!("chat message sent to mailbox"),
            Err(e) => println!("error sending to mailbox {:?}", e),
        };
    } else {
        error!("Failed to login, didn't send chat");
    }
    sleep(Duration::from_secs(1));
}

fn init_tests(incoming_socket_path: PathBuf, sender: Sender<PacketType>) {
    init_logger();
    let outgoing_socket_path = NamedTempFile::new()
        .expect("Failed to create temp file")
        .path()
        .to_path_buf();
    let outgoing_socket_path_clone = outgoing_socket_path.clone();
    let outgoing_socket_path_clone2 = outgoing_socket_path.clone();

    let incoming_socket_path_clone = incoming_socket_path.clone();

    println!("Starting outgoing UDS listener");

    std::thread::spawn(move || {
        let rt = tokio::runtime::Runtime::new().expect("Failed to create Tokio runtime");
        rt.block_on(async {
            listen_for_server_events(outgoing_socket_path_clone, sender).await;
        });
    });

    std::thread::spawn(|| {
        System::new().block_on(async {
            match initialize(incoming_socket_path_clone, outgoing_socket_path_clone2).await {
                Ok(handle) => {
                    match handle.await {
                        Ok(()) => info!("Listener exited successfully!"),
                        Err(e) => error!("Listener exited with error {:?}", e),
                    };
                }
                Err(err) => {
                    error!("Failed to start client: {:?}", err);
                }
            }
        });
    });
}

fn init_logger() {
    let _ = env_logger::builder()
        .filter(None, LevelFilter::Info)
        .is_test(true)
        .try_init();
}

/// helper function for building url. may be unnescecary
fn build_test_url(url: &str, port: u16) -> String {
    let mut url_string = "".to_owned();
    url_string.push_str(url);
    url_string.push(':');
    url_string.push_str(&port.to_string());
    println!("url string {}", url_string);
    url_string
}

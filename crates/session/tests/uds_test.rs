use actix::System;
use crossbeam_channel::unbounded;
use log::{error, info, LevelFilter};
use tempfile::NamedTempFile;

use metaverse_messages::chat_from_viewer::{ChatFromViewer, ClientChatType};
use metaverse_messages::login_system::login::Login;
use metaverse_messages::packet::Packet;
use metaverse_session::client_subscriber::listen_for_server_events;
use metaverse_session::initialize::initialize;
use uuid::Uuid;

use std::os::unix::net::UnixDatagram;
use std::thread::sleep;
use std::time::Duration;

#[test]
fn test_initialize() {
    init_logger();
    let incoming_socket_path = NamedTempFile::new()
        .expect("Failed to create temp file")
        .path()
        .to_path_buf();
    let outgoing_socket_path = NamedTempFile::new()
        .expect("Failed to create temp file")
        .path()
        .to_path_buf();
    let outgoing_socket_path_clone = outgoing_socket_path.clone();
    let outgoing_socket_path_clone2 = outgoing_socket_path.clone();

    let incoming_socket_path_clone = incoming_socket_path.clone();

    println!("Starting outgoing UDS listener");
    let (sender, _) = unbounded();

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

    // wait for the mailbox to be ready. This can be done in a better way.
    sleep(Duration::from_secs(2));
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

    sleep(Duration::from_secs(2));
    let chat_message = Packet::new_chat_from_viewer(ChatFromViewer {
        agent_id: Uuid::new_v4(),
        session_id: Uuid::new_v4(),
        message: "hello".to_string(),
        message_type: ClientChatType::Normal,
        channel: 0,
    })
    .to_bytes();
    let client_socket = UnixDatagram::unbound().unwrap();
    match client_socket.send_to(&chat_message, &incoming_socket_path) {
        Ok(_) => println!("message sent from mailbox"),
        Err(e) => println!("error sending from mailbox {:?}", e),
    };

    sleep(Duration::from_secs(5));
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

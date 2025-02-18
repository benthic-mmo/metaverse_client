use log::{error, info, LevelFilter};

use metaverse_messages::models::chat_from_viewer::{ChatFromViewer, ClientChatType};
use metaverse_messages::models::login::Login;
use metaverse_messages::models::packet::Packet;
use metaverse_session::initialize::initialize;
use metaverse_session::listener_util;
use tokio::time::{sleep, Duration};
use uuid::Uuid;

use std::fs;
use std::path::PathBuf;
use tokio::net::UnixDatagram;

#[actix_rt::test]
async fn test_initialize() {
    init_logger();
    let incoming_socket_path = PathBuf::from("/tmp/metaverse_incoming.sock");
    let outgoing_socket_path = PathBuf::from("/tmp/metaverse_outgoing.sock");
    let outgoing_socket_path_clone = outgoing_socket_path.clone();

    println!("starting outgoing UDS listener");
    tokio::spawn(async move {
        listener_util::listen(outgoing_socket_path_clone).await;
    });

    match initialize(incoming_socket_path.clone(), outgoing_socket_path.clone()).await {
        Ok(_) => {
            println!("mailbox initialized")
        }
        Err(e) => {
            println!("error :( {:?}", e)
        }
    };

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
    match client_socket.send_to(&message, &incoming_socket_path).await {
        Ok(_) => println!("message sent from mailbox"),
        Err(e) => println!("error sending from mailbox {:?}", e),
    };

    sleep(Duration::from_secs(2)).await;
    let chat_message = Packet::new_chat_from_viewer(ChatFromViewer {
        agent_id: Uuid::new_v4(),
        session_id: Uuid::new_v4(),
        message: "hello".to_string(),
        message_type: ClientChatType::Normal,
        channel: 0,
    })
    .to_bytes();
    let client_socket = UnixDatagram::unbound().unwrap();
    match client_socket
        .send_to(&chat_message, &incoming_socket_path)
        .await
    {
        Ok(_) => println!("message sent from mailbox"),
        Err(e) => println!("error sending from mailbox {:?}", e),
    };

    sleep(Duration::from_secs(3)).await;
    if incoming_socket_path.exists() {
        println!("Removing existing socket file");
        if let Err(e) = fs::remove_file(incoming_socket_path) {
            eprintln!("Failed to remove socket file: {}", e);
        }
    }
    if outgoing_socket_path.exists() {
        println!("Removing existing socket file");
        if let Err(e) = fs::remove_file(outgoing_socket_path) {
            eprintln!("Failed to remove socket file: {}", e);
        }
    }
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

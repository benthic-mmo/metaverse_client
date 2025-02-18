// this is for your client to listen on the data coming out of the server.
// import this and use directly, or modify to suit your own needs.

use metaverse_login::models::login_response::LoginResponse;
use std::{collections::HashMap, path::PathBuf};
use tokio::net::UnixDatagram;

use log::{error, info};

use crate::mailbox::TriggerSend;

pub async fn listen(socket_path: PathBuf) {
    let socket = UnixDatagram::bind(socket_path.clone()).unwrap();
    let mut message_store: HashMap<u16, String> = HashMap::new();
    info!(
        "Test client listening to outgoing UDS on: {:?}",
        socket_path
    );
    loop {
        let mut buf = [0u8; 1024];
        match socket.recv_from(&mut buf).await {
            Ok((n, _)) => {
                if let Ok(received_chunk) = TriggerSend::from_bytes(&buf[..n]) {
                    info!(
                        "Received chunk {}/{} for message type: {}",
                        received_chunk.sequence_number,
                        received_chunk.total_packet_number,
                        received_chunk.message_type
                    );

                    // Store the chunk
                    message_store
                        .entry(received_chunk.sequence_number)
                        .or_insert_with(String::new)
                        .push_str(&received_chunk.message);
                    // Check if we have all chunks
                    if message_store.len() == received_chunk.total_packet_number as usize {
                        let mut full_message = String::new();
                        for i in 0..received_chunk.total_packet_number {
                            if let Some(chunk) = message_store.remove(&i) {
                                full_message.push_str(&chunk);
                            } else {
                                error!("Missing chunk {} for message reconstruction", i);
                                return;
                            }
                        }
                        // we can do better than parsing this string but it's good enough for now
                        // After receiving the full message, check the message type and deserialize if needed
                        if received_chunk.message_type == "LoginResponse" {
                            match serde_json::from_str::<LoginResponse>(&full_message) {
                                Ok(login_response) => {
                                    info!(
                                        "Successfully deserialized LoginResponse: {:?}",
                                        login_response
                                    );
                                    // Now you can use the `login_response` object
                                }
                                Err(e) => {
                                    error!("Failed to deserialize LoginResponse: {:?}", e);
                                }
                            }
                        }
                    }
                } else {
                    info!("failed to deserialize the chunk :(")
                }
            }
            Err(e) => {
                error!("outgoing Failed to read buffer {}", e)
            }
        }
    }
}

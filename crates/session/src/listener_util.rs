// this is for your client to listen on the data coming out of the server.
// import this and use directly, or modify to suit your own needs.

use crossbeam_channel::Sender;
use metaverse_messages::login::login_response::LoginResponse;
use std::os::unix::net::UnixDatagram;
use std::{collections::HashMap, path::PathBuf};

use log::{error, info};

use crate::mailbox::TriggerSend;

pub async fn client_listen(socket_path: PathBuf, sender: Sender<LoginResponse>) {
    let socket = UnixDatagram::bind(socket_path).unwrap();
    let mut message_store: HashMap<u16, String> = HashMap::new();
    info!("Client listening to outgoing UDS on: {:?}", socket);
    loop {
        let mut buf = [0u8; 1024];
        match socket.recv_from(&mut buf) {
            Ok((n, _)) => {
                if let Ok(received_chunk) = TriggerSend::from_bytes(&buf[..n]) {
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
                                    {
                                        match sender.send(login_response) {
                                                    Ok(()) => info!("sent LoginResponse event to the login response handler"),
                                                    Err(e) => error!("failed to send to mspc {:?}", e)
                                                };
                                    };
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

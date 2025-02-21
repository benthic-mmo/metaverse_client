use crossbeam_channel::Sender;
use metaverse_messages::login_system::login_response::LoginResponse;
use std::os::unix::net::UnixDatagram;
use std::{collections::HashMap, path::PathBuf};

use log::{info, warn};

use crate::mailbox::UiMessage;

/// This is for your client to listen on the data coming out of the server.
/// import this and use directly, or modify to suit your own needs.
/// By default you can use this to run in the background, and subscribe to the outgoing events
/// using crossbeam.
/// Events will be sent to the crossbeam sender, and received in your UI with the crossbeam
/// receiver.
/// ```rust
/// use metaverse_session::client_subscriber::listen_for_server_events;
/// use crossbeam_channel::{unbounded, Receiver, Sender};
/// use std::thread::spawn;
/// use tokio::runtime::Runtime;
/// use tempfile::NamedTempFile;
///
/// let (sender, receiver) = unbounded();
/// let outgoing_socket_path = NamedTempFile::new()
///     .expect("Failed to create temp file")
///     .path()
///     .to_path_buf();
/// spawn(move || {
///     let rt = Runtime::new().expect("Failed to create Tokio runtime");
///     rt.block_on(async {
///         listen_for_server_events(outgoing_socket_path, sender).await;
///     });
/// });
/// ```
///
/// In your client you can use something like this to handle events.
/// just remember to keep your crossbeam_channels as shared resources.
/// for example, create the crossbeam_channels in your main function, start your thread, save that
/// channel to a global variable, and then run handle_queue once per frame.
/// ```text
/// use crossbeam_channel::Receiver;
///
///fn handle_queue(receiver: Receiver) {
///    while let Ok(event) = receiver.try_recv() {
///         somefunction(event)
///     }
///}
///```
pub async fn listen_for_server_events(socket_path: PathBuf, sender: Sender<LoginResponse>) {
    let socket = UnixDatagram::bind(socket_path).unwrap();
    let mut message_store: HashMap<u16, String> = HashMap::new();
    info!("UI listening for server events on UDS: {:?}", socket);
    loop {
        let mut buf = [0u8; 1024];
        match socket.recv_from(&mut buf) {
            Ok((n, _)) => {
                if let Ok(received_chunk) = UiMessage::from_bytes(&buf[..n]) {
                    message_store
                        .entry(received_chunk.sequence_number)
                        .or_default()
                        .push_str(&received_chunk.message);
                    // Check if we have all chunks
                    if message_store.len() == received_chunk.total_packet_number as usize {
                        let mut full_message = String::new();
                        for i in 0..received_chunk.total_packet_number {
                            if let Some(chunk) = message_store.remove(&i) {
                                full_message.push_str(&chunk);
                            } else {
                                warn!("Missing chunk {} for message reconstruction", i);
                                return;
                            }
                        }
                        // TODO: right now this function ONLY works on LoginResponses.
                        // that's not good. This is going to become much more generic very soon.
                        // we can do better than parsing this string but it's good enough for now
                        // After receiving the full message, check the message type and deserialize if needed
                        if received_chunk.message_type == "LoginResponse" {
                            match serde_json::from_str::<LoginResponse>(&full_message) {
                                Ok(login_response) => {
                                    {
                                        match sender.send(login_response) {
                                                    Ok(()) => info!("sent LoginResponse event to the login response handler"),
                                                    Err(e) => warn!("failed to send to mspc {:?}", e)
                                                };
                                    };
                                }
                                Err(e) => {
                                    warn!(
                                        "UI failed to deserialize LoginResponse from server: {:?}",
                                        e
                                    );
                                }
                            }
                        }
                    }
                } else {
                    warn!("UI failed to deserialize the packet chunk from server")
                }
            }
            Err(e) => {
                warn!("UI Failed to read buffer {}", e)
            }
        }
    }
}

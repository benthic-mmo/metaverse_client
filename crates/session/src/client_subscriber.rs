use crossbeam_channel::Sender;
use log::error;
use metaverse_messages::chat_from_simulator::ChatFromSimulator;
use metaverse_messages::coarse_location_update::CoarseLocationUpdate;
use metaverse_messages::login_system::login_response::LoginResponse;
use metaverse_messages::packet::PacketData;
use metaverse_messages::ui_events::UiEventTypes;
use std::os::unix::net::UnixDatagram;
use std::{collections::HashMap, path::PathBuf};

use log::{info, warn};

use crate::mailbox::UiMessage;

/// This stores the packet and the chunks for deserialization
pub struct PacketStore {
    /// the chunks that belong to that packet
    chunks: HashMap<u16, Vec<u8>>,
}

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
    let mut message_store: HashMap<u16, PacketStore> = HashMap::new();

    info!("UI listening for server events on UDS: {:?}", socket);
    loop {
        let mut buf = [0u8; 1024];
        match socket.recv_from(&mut buf) {
            Ok((n, _)) => {
                if let Some(received_chunk) = UiMessage::from_bytes(&buf[..n]) {
                    let packet_store =
                        message_store
                            .entry(received_chunk.packet_number)
                            .or_insert(PacketStore {
                                chunks: HashMap::new(),
                            });

                    packet_store
                        .chunks
                        .insert(received_chunk.sequence_number, received_chunk.message);

                    // Check if we have all chunks
                    if packet_store.chunks.len() == received_chunk.total_packet_number as usize {
                        let mut full_message = Vec::new();
                        for i in 0..received_chunk.total_packet_number {
                            if let Some(chunk) = packet_store.chunks.remove(&i) {
                                full_message.extend_from_slice(&chunk);
                            } else {
                                warn!("Missing chunk {} for message reconstruction", i);
                                return;
                            }
                        }
                        match received_chunk.message_type {
                            UiEventTypes::LoginResponseEvent => {
                                match serde_json::from_str::<LoginResponse>(
                                    &String::from_utf8(full_message).unwrap(),
                                ) {
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
                            UiEventTypes::ChatEvent => {
                                let _packet = match ChatFromSimulator::from_bytes(&full_message) {
                                    Ok(packet) => info!("parsed packet: {:?}", packet),
                                    Err(e) => {
                                        error!("failed to parse packet {:?}", e);
                                        continue;
                                    }
                                };
                            }
                            UiEventTypes::CoarseLocationUpdateEvent => {
                                let _packet = match CoarseLocationUpdate::from_bytes(&full_message)
                                {
                                    Ok(packet) => info!("parsed packet: {:?}", packet),
                                    Err(e) => {
                                        error!("failed to parse packet {:?}", e);
                                        continue;
                                    }
                                };
                            }
                            event => {
                                info!("{:?} not implemented yet", event)
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

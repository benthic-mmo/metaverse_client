use crossbeam_channel::Sender;
use metaverse_messages::packet_types::PacketType;
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
///         match event{
///             PacketType::LoginResponse(login_response) => {
///                 // handle the login response
///             }
///             PacketType::CoarseLocationUpdate(coarse_location_update) => {
///                 // handle the coarse location update
///             }
///             // etc for the rest of the event packets
///         }
///     }
///}
///```
pub async fn listen_for_server_events(socket_path: PathBuf, sender: Sender<PacketType>) {
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
                        // get the packet type and send that to the sender
                        if let Some(packet) = received_chunk
                            .message_type
                            .packet_type_from_bytes(&full_message)
                        {
                            if let Err(e) = sender.send(packet) {
                                warn!("Failed to send packet to UI: {:?}", e)
                            };
                        } else {
                            warn!("Client failed to send packet to UI")
                        };
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

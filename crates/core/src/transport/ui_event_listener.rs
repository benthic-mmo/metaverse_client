use crate::session::Mailbox;
use log::warn;
use metaverse_messages::packet::message::UIResponse;
use tokio::net::UdpSocket;

/// This is used to enable the core to listen to messages coming in from the UI.
/// Messages are sent to the core using UDP, and are deserializeable as UiResponse objects.
pub async fn listen_for_ui_messages(ui_to_core_socket: String, mailbox_addr: actix::Addr<Mailbox>) {
    let socket = UdpSocket::bind(ui_to_core_socket)
        .await
        .expect("Failed to bind to UDP socket");
    loop {
        let mut buf = [0u8; 1500];
        match socket.recv_from(&mut buf).await {
            Ok((n, _)) => {
                let event = match UIResponse::from_bytes(&buf[..n]) {
                    Ok(event) => event,
                    Err(e) => {
                        warn!("Failed to receive event {:?},", e);
                        continue;
                    }
                };
                mailbox_addr.do_send(event);
            }
            Err(e) => {
                warn!("Core failed to read buffer sent from UI {}", e)
            }
        }
    }
}

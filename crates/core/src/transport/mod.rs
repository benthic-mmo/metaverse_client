/// handles sending requests to HTTP endpoints on the server.
/// This includes logins, and capability endpoint requests.
pub mod http_handler;
/// Handles packets sent in from the server on the UDP endpoint.
/// This contains the main async loop that runs in the background, receiving packets from the
/// server, and differentiating which type of packet is being received, which will be used to notify
/// the mailbox and enable further handling.
pub mod udp_handler;
/// Handles packets sent in from the UI on the UI -> Core communication port.
/// This contains the main async loop that runs in the background, receiving packets from the UI,
/// allowing for the core to respond to UI events.
pub mod ui_event_listener;

use std::path::PathBuf;

use metaverse_session::initialize::initialize;

#[actix_rt::main]
async fn main() {
    // add a hash here so this doesn't ever get overwritten by two processes
    let incoming_socket_path = PathBuf::from("/tmp/metaverse_incoming.sock");
    let outgoing_socket_path = PathBuf::from("/tmp/metaverse_outgoing.sock");

    match initialize(incoming_socket_path, outgoing_socket_path).await {
        Ok(_) => {
            println!("succeeded")
        }
        Err(e) => {
            println!("error: {:?}", e)
        }
    };
}

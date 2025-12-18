use std::{fs::File, io::Read};

use metaverse_messages::http::login::simulator_login_protocol::SimulatorLoginProtocol;

#[test]
fn parse_login_failure() {
    let mut file = File::open("tests/data/simulator_login_protocol.txt").unwrap();
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer).unwrap();

    let serialized = String::from_utf8(buffer).unwrap();
    let response: SimulatorLoginProtocol = serde_json::from_str(&serialized).unwrap();
    let _llsd = response.to_llsd_map();
    let _xml = response.to_xmlrpc();
}

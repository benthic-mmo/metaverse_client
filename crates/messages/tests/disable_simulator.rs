use hex::FromHex;
use metaverse_messages::models::packet::Packet;

#[test]
fn test_disable_simulator() {
    let test_packet = match Vec::from_hex("400000000300ffff0098") {
        Ok(bytes) => bytes,
        Err(_) => panic!("failed"),
    };
    match Packet::from_bytes(&test_packet) {
        Ok(packet) => println!("Packet created successfully: {:?}", packet),
        Err(e) => eprintln!("Error creating packet: {}", e),
    }
}

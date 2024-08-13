use hex::FromHex;
use metaverse_messages::models::packet::Packet;

#[test]
fn test_coarse_location_update() {
    let test_packet = match Vec::from_hex("000000000100ff0600ffffffff00") {
        Ok(bytes) => bytes,
        Err(_) => panic!("failed"),
    };
    match Packet::from_bytes(&test_packet) {
        Ok(packet) => println!("Packet created successfully: {:?}", packet),
        Err(e) => eprintln!("Error creating packet: {}", e),
    }
}

#[test]
fn test_coarse_location_update_firestorm() {
    let test_packet = match Vec::from_hex("400000000300ffff0018") {
        Ok(bytes) => bytes,
        Err(_) => panic!("failed"),
    };
    match Packet::from_bytes(&test_packet) {
        Ok(packet) => println!("Packet created successfully: {:?}", packet),
        Err(e) => eprintln!("Error creating packet: {}", e),
    }
}

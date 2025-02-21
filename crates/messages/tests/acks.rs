use hex::FromHex;
use metaverse_messages::packet::Packet;

#[test]
fn test_acks_parse() {
    let test_packet = match Vec::from_hex("000000000000fffffffb0100000000") {
        Ok(bytes) => bytes,
        Err(_) => panic!("failed"),
    };
    match Packet::from_bytes(&test_packet) {
        Ok(packet) => println!("Packet created successfully: {:?}", packet),
        Err(e) => eprintln!("Error creating packet: {}", e),
    }
}

#[test]
fn test_acks_firestorm_parse() {
    let test_packet = match Vec::from_hex("000000000000fffffffb0101000000") {
        Ok(bytes) => bytes,
        Err(_) => panic!("failed"),
    };
    match Packet::from_bytes(&test_packet) {
        Ok(packet) => println!("Packet created successfully: {:?}", packet),
        Err(e) => eprintln!("Error creating packet: {}", e),
    }
}

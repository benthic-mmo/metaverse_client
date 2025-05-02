use hex::FromHex;
use metaverse_messages::packet::Packet;

#[test]
fn test_use_circuit_code_firestorm_parse() {
    let test_packet = match Vec::from_hex(
        "000000000000ffff000389c0e85824cd791211bf4b3ba4697896a707864a9dc18bb1044f4c68906b2cb608b2e197",
    ) {
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
    let test_packet = match Vec::from_hex(
        "400000000100ffff000384d7e147a7d76c2d81da467faadade6f6af2d2bd9dc18bb1044f4c68906b2cb608b2e197",
    ) {
        Ok(bytes) => bytes,
        Err(_) => panic!("failed"),
    };
    match Packet::from_bytes(&test_packet) {
        Ok(packet) => println!("Packet created successfully: {:?}", packet),
        Err(e) => eprintln!("Error creating packet: {}", e),
    }
}

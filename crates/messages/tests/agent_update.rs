use hex::FromHex;
use metaverse_messages::packet::Packet;

#[test]
fn test_agent_update() {
    let test_packet = match Vec::from_hex("800000110c0004320dff8a7a594720a0f75a8df3698d9a224ecaea372d4d318b644805966418e500
08f404353f0008f404353f0004430002fa42bce33440000471f5793f9f255dbe000280bf000c9f255d3e71f5793f0003430005") {
        Ok(bytes) => bytes,
        Err(_) => panic!("failed"),
    };
    match Packet::from_bytes(&test_packet) {
        Ok(packet) => println!("Packet created successfully: {:?}", packet),
        Err(e) => eprintln!("Error creating packet: {}", e),
    }
}

use metaverse_messages::packet::{packet::Packet, packet_types::PacketType};

const TEST_PACKET: [u8; 46] = [
    0, 0, 0, 0, 0, 0, 255, 255, 0, 3, 137, 192, 232, 88, 36, 205, 121, 18, 17, 191, 75, 59, 164,
    105, 120, 150, 167, 7, 134, 74, 157, 193, 139, 177, 4, 79, 76, 104, 144, 107, 44, 182, 8, 178,
    225, 151,
];

const CODE: u32 = 1491648649;

#[test]
fn test_circuit_code() {
    let packet = match Packet::from_bytes(&TEST_PACKET) {
        Ok(packet) => packet,
        Err(e) => panic!("Failed to create packet: {:?}", e),
    };
    match packet.body {
        PacketType::CircuitCode(packet) => {
            assert!(packet.code == CODE);
        }
        _ => {}
    }
}

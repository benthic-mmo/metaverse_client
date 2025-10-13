use metaverse_messages::{packet::packet::Packet, udp::core::disable_simulator::DisableSimulator};

const TEST_PACKET: [u8; 10] = [64, 0, 0, 0, 0, 0, 255, 255, 0, 152];
#[test]
fn test_disable_simulator() {
    let packet = match Packet::from_bytes(&TEST_PACKET) {
        Ok(packet) => packet,
        Err(e) => panic!("Error creating packet: {}", e),
    };
    assert!(Packet::new_disable_simulator(DisableSimulator {}).to_bytes() == packet.to_bytes())
}

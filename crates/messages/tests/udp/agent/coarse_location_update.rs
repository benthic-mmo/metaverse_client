use metaverse_messages::packet::{packet::Packet, packet_types::PacketType};

const TEST_PACKET: [u8; 13] = [0, 0, 0, 0, 1, 0, 255, 6, 0, 255, 255, 255, 255];

#[test]
fn test_coarse_location_update() {
    let packet = match Packet::from_bytes(&TEST_PACKET) {
        Ok(packet) => packet,
        Err(e) => panic!("Error creating packet: {}", e),
    };
    match packet.body {
        PacketType::CoarseLocationUpdate(packet) => {
            assert!(-1 == packet.you);
        }
        _ => {}
    }
}

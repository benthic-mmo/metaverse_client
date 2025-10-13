use metaverse_messages::packet::{packet::Packet, packet_types::PacketType};
use uuid::{uuid, Uuid};

const TEST_PACKET: [u8; 129] = [
    0, 0, 0, 0, 1, 0, 4, 251, 46, 85, 65, 102, 186, 64, 25, 165, 222, 232, 185, 40, 107, 9, 20,
    161, 18, 208, 184, 76, 102, 66, 73, 138, 172, 228, 84, 114, 189, 140, 49, 0, 0, 128, 63, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 128, 63, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 128, 63, 0, 0, 128, 63, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 128, 63, 0, 0, 0, 0, 0, 0, 72, 67, 0, 0, 0, 0, 0,
];
const DEFAULT_USER_ID: Uuid = uuid!("fb2e5541-66ba-4019-a5de-e8b9286b0914");

#[test]
fn test_agent_update() {
    let packet = match Packet::from_bytes(&TEST_PACKET) {
        Ok(p) => p,
        Err(e) => panic!("Error creating packet: {}", e),
    };
    match packet.body {
        PacketType::AgentUpdate(update) => {
            assert!(update.agent_id == DEFAULT_USER_ID)
        }
        _ => {}
    }
}

use metaverse_messages::{
    chat::{chat_from_viewer::ChatFromViewer, ChatType},
    packet::{packet::Packet, packet_types::PacketType},
};
use uuid::{uuid, Uuid};

const TEST_PACKET: [u8; 51] = [
    64, 0, 0, 17, 6, 0, 255, 255, 0, 80, 50, 13, 255, 138, 122, 89, 71, 32, 160, 247, 90, 141, 243,
    105, 141, 154, 34, 78, 202, 234, 55, 45, 77, 49, 139, 100, 72, 5, 150, 100, 24, 229, 2, 0, 97,
    0, 1, 0, 0, 0, 0,
];

const SESSION_ID: Uuid = uuid!("224ecaea-372d-4d31-8b64-4805966418e5");
const AGENT_ID: Uuid = uuid!("320dff8a-7a59-4720-a0f7-5a8df3698d9a");

#[test]
fn test_chat_from_viewer() {
    let packet = match Packet::from_bytes(&TEST_PACKET) {
        Ok(p) => p,
        Err(e) => panic!("Error creating packet: {}", e),
    };
    match packet.body {
        PacketType::ChatFromViewer(packet) => {
            assert!(packet.agent_id == AGENT_ID);
            assert!(packet.message == "a\0")
        }
        _ => {
            panic!("Packet improperly decoded")
        }
    }
}

#[test]
fn create_chat_from_viewer() {
    let packet = Packet::new_chat_from_viewer(ChatFromViewer {
        agent_id: AGENT_ID,
        session_id: SESSION_ID,
        channel: 1,
        message: "a".to_string(),
        message_type: ChatType::Normal,
    });
    match packet.body {
        PacketType::ChatFromViewer(packet) => {
            assert!(packet.message == "a");
            assert!(packet.agent_id == AGENT_ID);
            assert!(packet.session_id == SESSION_ID)
        }
        _ => {
            panic!("Packet improperly decoded")
        }
    }
}

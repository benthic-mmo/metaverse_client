use hex::FromHex;
use metaverse_messages::models::{
    chat_from_viewer::{ChatFromViewer, ClientChatType},
    packet::Packet,
};
use uuid::uuid;

#[test]
fn test_chat_from_viewer() {
    let test_packet = match Vec::from_hex("400000110600ffff0050320dff8a7a594720a0f75a8df3698d9a224ecaea372d4d318b644805966418e5020061000100000000") {
        Ok(bytes) => bytes,
        Err(_) => panic!("failed"),
    };
    match Packet::from_bytes(&test_packet) {
        Ok(packet) => println!("Packet created successfully: {:?}", packet),
        Err(e) => eprintln!("Error creating packet: {}", e),
    }
}

#[test]
fn create_chat_from_viewer() {
    let packet = Packet::new_chat_from_viewer(ChatFromViewer {
        agent_id: uuid!("320dff8a-7a59-4720-a0f7-5a8df3698d9a"),
        session_id: uuid!("224ecaea-372d-4d31-8b64-4805966418e5"),
        channel: 1,
        message: "a".to_string(),
        message_type: ClientChatType::Normal,
    });
    println!("{:?}", packet);
    println!("{:?}", hex::encode(packet.to_bytes()));
    println!("\"400000110600ffff0050320dff8a7a594720a0f75a8df3698d9a224ecaea372d4d318b644805966418e5020061000100000000\"")
}

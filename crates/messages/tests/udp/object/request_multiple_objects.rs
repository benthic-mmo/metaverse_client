use metaverse_messages::{
    packet::{
        packet::{Packet, PacketData},
        packet_types::PacketType,
    },
    udp::object::request_multiple_objects::{CacheMissType, RequestMultipleObjects},
};
use uuid::Uuid;

const PACKET_DATA: [u8; 61] = [
    64, 0, 0, 0, 22, 0, 255, 3, 127, 60, 38, 51, 229, 96, 70, 31, 191, 207, 157, 20, 141, 88, 229,
    164, 7, 166, 254, 147, 198, 202, 68, 57, 188, 58, 202, 189, 59, 24, 6, 166, 4, 0, 76, 3, 58,
    14, 0, 77, 3, 58, 14, 0, 78, 3, 58, 14, 0, 79, 3, 58, 14,
];
const PACKET_DATA2: [u8; 61] = [
    64, 0, 0, 0, 28, 0, 255, 3, 127, 60, 38, 51, 229, 96, 70, 31, 191, 207, 157, 20, 141, 88, 229,
    164, 34, 173, 177, 9, 69, 87, 70, 187, 134, 230, 193, 36, 153, 137, 27, 148, 4, 0, 109, 17,
    220, 18, 0, 110, 17, 220, 18, 0, 111, 17, 220, 18, 0, 112, 17, 220, 18,
];

#[test]
fn test_request_multiple_objects() {
    match Packet::from_bytes(&PACKET_DATA2).unwrap().body {
        PacketType::RequestMultipleObjects(data) => {
            println!("{:?}", data)
        }
        _ => {}
    }
    let created_packet = Packet::new_request_multiple_objects(RequestMultipleObjects {
        agent_id: Uuid::nil(),
        session_id: Uuid::nil(),
        requests: vec![(CacheMissType::Normal, 1), (CacheMissType::Normal, 2)],
    });
    match Packet::from_bytes(&created_packet.to_bytes()).unwrap().body {
        PacketType::RequestMultipleObjects(data) => {
            assert_eq!(data.agent_id, Uuid::nil());
            assert_eq!(data.session_id, Uuid::nil());
            assert_eq!(data.requests.len(), 2);
            assert_eq!(data.requests[0].1, 1);
        }
        _ => {}
    }
}

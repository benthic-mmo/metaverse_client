use metaverse_messages::{
    packet::packet::PacketData, udp::object::object_update_cached::ObjectUpdateCached,
};

const BODY_BYTES: [u8; 60] = [
    0, 232, 3, 0, 0, 232, 3, 0, 255, 255, 4, 3, 132, 194, 53, 44, 230, 253, 175, 60, 9, 2, 16, 4,
    132, 194, 53, 246, 30, 254, 175, 60, 9, 2, 16, 5, 132, 194, 53, 116, 35, 138, 182, 60, 9, 2,
    16, 6, 132, 194, 53, 24, 4, 87, 138, 182, 60, 9, 2, 16,
];

#[test]
fn test_object_update_cached() {
    ObjectUpdateCached::from_bytes(&BODY_BYTES).unwrap();
}

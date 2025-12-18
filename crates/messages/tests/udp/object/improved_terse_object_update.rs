use metaverse_messages::{
    packet::packet::PacketData,
    udp::object::improved_terse_object_update::ImprovedTerseObjectUpdate,
};

const BODY_BYTES: [u8; 74] = [
    0, 232, 3, 0, 0, 232, 3, 0, 255, 255, 1, 60, 51, 132, 194, 53, 0, 1, 136, 111, 35, 60, 152, 0,
    143, 189, 198, 92, 127, 63, 76, 44, 139, 65, 56, 65, 0, 67, 139, 125, 249, 66, 76, 202, 203,
    65, 255, 127, 255, 127, 255, 127, 255, 127, 255, 127, 255, 127, 255, 127, 255, 127, 181, 130,
    247, 255, 255, 127, 255, 127, 255, 127, 0, 0,
];

#[test]
fn test_improved_terse_object_update() {
    ImprovedTerseObjectUpdate::from_bytes(&BODY_BYTES).unwrap();
}

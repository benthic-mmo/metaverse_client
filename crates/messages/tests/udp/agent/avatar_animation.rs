use metaverse_messages::{
    packet::packet::PacketData, udp::agent::avatar_animation::AvatarAnimation,
};

const EXAMPLE_ANIM: [u8; 39] = [
    127, 60, 38, 51, 229, 96, 70, 31, 191, 207, 157, 20, 141, 88, 229, 164, 1, 36, 8, 254, 158,
    223, 29, 29, 125, 244, 255, 19, 132, 250, 123, 53, 15, 1, 0, 0, 0, 0, 0,
];

#[test]
fn test_avatar_animation() {
    AvatarAnimation::from_bytes(&EXAMPLE_ANIM).unwrap();
}

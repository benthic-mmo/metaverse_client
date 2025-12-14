use metaverse_messages::{
    packet::packet::Packet,
    udp::core::agent_throttle::{AgentThrottle, ThrottleData},
};
use uuid::Uuid;

#[test]
fn test_agent_throttle() {
    let packet = Packet::new_agent_throttle(AgentThrottle {
        agent_id: Uuid::nil(),
        session_id: Uuid::nil(),
        circuit_code: 1234,
        gen_counter: 0,
        throttles: ThrottleData::new_total(1_536_000.0),
    });

    let packet_bytes = packet.to_bytes();

    println!("{:?}, {:?}", packet_bytes, packet_bytes.len());
    let parsed = Packet::from_bytes(&packet.to_bytes());
    println!("{:?}", parsed);
}

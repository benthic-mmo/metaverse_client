use metaverse_messages::models::use_circuit_code::*;
use uuid::Uuid;

//TODO: this test really sucks tbh, no assert it just checks for a crash
#[test]
fn test_use_circuit_code_packet() {
    let code = create_use_circuit_code(
        697482820,
        Uuid::parse_str("11111111-1111-0000-0000-000100bba000").unwrap(),
        Uuid::parse_str("6ac2e761-f490-4122-bf6c-7ad8fbb17002").unwrap(),
    );

    let output = create_use_circuit_code_packet(code);
    println!("{:?}", output);
}

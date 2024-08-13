use hex::FromHex;
use metaverse_messages::models::{
    header::Header,
    packet_types::PacketType,
};

#[test]
fn test_acks_parse() {
    let test_packet = match Vec::from_hex("000000000000fffffffb0100000000") {
        Ok(bytes) => bytes,
        Err(_) => panic!("failed"),
    };
    println!("packet bytes are: {:?}", test_packet);
    let test_header = Header::try_from_bytes(&test_packet).unwrap();
    println!("header is {:?}", test_header);
    if test_header.size.unwrap_or(0) < test_packet.len() {
        let slice = &test_packet[test_header.size.unwrap()..];
        println!("data is {:?}", slice);
    } else {
        println!("Index out of bounds for slicing");
    }
    let body_bytes = &test_packet[test_header.size.unwrap_or(0)..];
    let body = match PacketType::from_id(test_header.id, body_bytes) {
        Ok(body) => body,
        Err(e) => {
            println!("Error parsing packet body: {:?}", e);
            return;
        }
    };
    println!("Body Received: {:?}", body);
}

#[test]
fn test_acks_firestorm_parse() {
    let test_packet = match Vec::from_hex("000000000000fffffffb0101000000") {
        Ok(bytes) => bytes,
        Err(_) => panic!("failed"),
    };
    println!("packet bytes are: {:?}", test_packet);
    let test_header = Header::try_from_bytes(&test_packet).unwrap();
    println!("header is {:?}", test_header);
    if test_header.size.unwrap_or(0) < test_packet.len() {
        let slice = &test_packet[test_header.size.unwrap()..];
        println!("data is {:?}", slice);
    } else {
        println!("Index out of bounds for slicing");
    }
    let body_bytes = &test_packet[test_header.size.unwrap_or(0)..];
    let body = match PacketType::from_id(test_header.id, body_bytes) {
        Ok(body) => body,
        Err(e) => {
            println!("Error parsing packet body: {:?}", e);
            return;
        }
    };
    println!("Body Received: {:?}", body);
}

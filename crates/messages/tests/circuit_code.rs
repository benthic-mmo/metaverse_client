use hex::FromHex;
use metaverse_messages::models::{header::Header, packet_types::PacketType};

#[test]
fn test_use_circuit_code_firestorm_parse() {
    let test_packet = match Vec::from_hex("000000000000ffff000389c0e85824cd791211bf4b3ba4697896a707864a9dc18bb1044f4c68906b2cb608b2e197") {
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
    let body = match PacketType::from_id(test_header.id, test_header.frequency, body_bytes) {
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
    let test_packet = match Vec::from_hex("400000000100ffff000384d7e147a7d76c2d81da467faadade6f6af2d2bd9dc18bb1044f4c68906b2cb608b2e197") {
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
    let body = match PacketType::from_id(test_header.id, test_header.frequency, body_bytes) {
        Ok(body) => body,
        Err(e) => {
            println!("Error parsing packet body: {:?}", e);
            return;
        }
    };
    println!("Body Received: {:?}", body);
}

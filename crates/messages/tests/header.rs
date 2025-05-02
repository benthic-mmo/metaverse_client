use hex::FromHex;
use metaverse_messages::header::{Header, PacketFrequency};

#[test]
fn test_header_for_acks() {
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
}

#[test]
fn test_header_for_circuit_code() {
    let test_packet = match Vec::from_hex(
        "000000000000ffff000389c0e85824cd791211bf4b3ba4697896a707864a9dc18bb1044f4c68906b2cb608b2e197",
    ) {
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
}

#[test]
fn test_header_for_coarse_location_update() {
    let test_packet = match Vec::from_hex("000000000100ff0600ffffffff00") {
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
}
#[test]
fn test_header_tofrom_bytes() {
    let test_header = Header {
        reliable: true,
        resent: false,
        zerocoded: false,
        appended_acks: false,
        sequence_number: 0,
        id: 42069,
        frequency: PacketFrequency::Low,
        ack_list: None,
        size: None,
    };

    let header_bytes = test_header.to_bytes();
    let header_from_bytes = Header::try_from_bytes(&header_bytes).unwrap();
    let header_back_to_bytes = header_from_bytes.to_bytes();
    assert!(header_bytes == header_back_to_bytes);

    let test_header = Header {
        reliable: true,
        resent: false,
        zerocoded: false,
        appended_acks: false,
        sequence_number: 0,
        id: 42069,
        frequency: PacketFrequency::Medium,
        ack_list: None,
        size: None,
    };

    let header_bytes = test_header.to_bytes();
    let header_from_bytes = Header::try_from_bytes(&header_bytes).unwrap();
    let header_back_to_bytes = header_from_bytes.to_bytes();
    assert!(header_bytes == header_back_to_bytes);

    let test_header = Header {
        reliable: true,
        resent: false,
        zerocoded: false,
        appended_acks: false,
        sequence_number: 0,
        id: 42069,
        frequency: PacketFrequency::High,
        ack_list: None,
        size: None,
    };

    let header_bytes = test_header.to_bytes();
    let header_from_bytes = Header::try_from_bytes(&header_bytes).unwrap();
    let header_back_to_bytes = header_from_bytes.to_bytes();

    assert!(header_bytes == header_back_to_bytes);
}

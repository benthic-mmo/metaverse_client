use metaverse_messages::packet::header::{Header, PacketFrequency};

const TEST_PACKET: [u8; 15] = [0, 0, 0, 0, 0, 0, 255, 255, 255, 251, 1, 0, 0, 0, 0];

#[test]
fn test_header_for_acks() {
    let test_header = Header::try_from_bytes(&TEST_PACKET).unwrap();
    assert!(test_header.reliable == false);
    assert!(test_header.sequence_number == 0);
}

#[test]
fn test_header_to_from_bytes() {
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
}

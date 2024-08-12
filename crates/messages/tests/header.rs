use metaverse_messages::models::header::{Header, PacketFrequency};

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
    };

    let header_bytes = test_header.to_bytes();
    let header_from_bytes = Header::try_from_bytes(&header_bytes).unwrap();
    let header_back_to_bytes = header_from_bytes.to_bytes();

    assert!(header_bytes == header_back_to_bytes);
}

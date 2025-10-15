use metaverse_messages::http::item::ItemData;
use std::{fs::File, io::Read};

#[test]
fn parse_login_response() {
    let mut file = File::open("tests/data/item2.txt").unwrap();
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer).unwrap();

    let llsd = ItemData::from_bytes(&buffer);
    assert_eq!(llsd.unwrap().version, 22);
}

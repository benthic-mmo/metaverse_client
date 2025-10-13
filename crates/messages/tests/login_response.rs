use metaverse_messages::ui::login::{login_errors::Reason, login_response::LoginResponse};
use std::{fs::File, io::Read};
use uuid::uuid;
use uuid::Uuid;

#[test]
fn parse_login_response() {
    let mut file = File::open("tests/data/login_response_data.txt").unwrap();
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer).unwrap();

    let xml_string = String::from_utf8(buffer).unwrap();
    let response = LoginResponse::from_xml(&xml_string).unwrap();
    assert_eq!(response.first_name, "Justin".to_string());
    assert_eq!(
        response.inventory_skeleton.unwrap()[0].folder_id,
        uuid!("004d663b-9980-46ae-8559-bb60e9d67d28")
    );
}
#[test]
fn parse_login_response_live() {
    let mut file = File::open("tests/data/login_response_2.txt").unwrap();
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer).unwrap();

    let xml_string = String::from_utf8(buffer).unwrap();
    let response = LoginResponse::from_xml(&xml_string).unwrap();
    assert_eq! {
        response.seed_capability , Some("http://192.168.0.28:9000/CAPS/6d678be4-de3a-4745-a008-58f0d0470b2a0000/".to_string())
    };
    assert_eq! {response.circuit_code, 1310932211};
}
#[test]
fn parse_login_failure() {
    let mut file = File::open("tests/data/login_error.txt").unwrap();
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer).unwrap();

    let xml_string = String::from_utf8(buffer).unwrap();
    let response = LoginResponse::from_xml(&xml_string);
    match response {
        Err(e) => {
            assert!(e.reason == Reason::Presence)
        }
        fail => panic!("Failed to generate login error{:?}", fail),
    }
}

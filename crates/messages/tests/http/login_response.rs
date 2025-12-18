use metaverse_messages::http::login::{
    login_error::Reason,
    login_response::{LoginResponse, LoginStatus},
};
use std::{fs::File, io::Read};
use uuid::uuid;

#[test]
fn parse_login_response() {
    let mut file = File::open("tests/data/login_response_data.txt").unwrap();
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer).unwrap();

    let xml_string = String::from_utf8(buffer).unwrap();
    let response = LoginResponse::from_xmlrpc(&xml_string).unwrap();
    match response {
        LoginStatus::Success(response) => {
            assert_eq!(response.first_name, "Justin".to_string());
            assert_eq!(
                response.inventory_skeleton.unwrap()[0].folder_id,
                uuid!("004d663b-9980-46ae-8559-bb60e9d67d28")
            );
            assert_eq!(
                response.inventory_root,
                Some(uuid!("37c4cfe3-ea39-4ef7-bda3-bee73bd46d95"))
            );
            assert_eq!(
                response.inventory_lib_owner,
                Some(uuid!("11111111-1111-0000-0000-000100bba000"))
            );
        }
        _ => panic!("login response failed"),
    }
}

#[test]
fn parse_login_response_live() {
    let mut file = File::open("tests/data/login_response_2.txt").unwrap();
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer).unwrap();

    let xml_string = String::from_utf8(buffer).unwrap();
    let response = LoginResponse::from_xmlrpc(&xml_string).unwrap();
    match response {
        LoginStatus::Success(response) => {
            assert_eq! {
                response.seed_capability , Some("http://192.168.0.28:9000/CAPS/6d678be4-de3a-4745-a008-58f0d0470b2a0000/".to_string())
            };
            assert_eq! {response.circuit_code, 1310932211};
        }
        _ => panic!("login response failed"),
    }
}
#[test]
fn parse_login_failure() {
    let mut file = File::open("tests/data/login_error.txt").unwrap();
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer).unwrap();

    let xml_string = String::from_utf8(buffer).unwrap();
    let response = LoginResponse::from_xmlrpc(&xml_string).unwrap();
    match response {
        LoginStatus::Failure(response) => {
            assert_eq!(response.reason, Reason::Presence)
        }
        _ => panic!("Login response failed to parse"),
    }
}

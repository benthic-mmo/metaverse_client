use crate::login_system::errors::{LoginError, Reason};
use crate::login_system::login_response::LoginResponse;
use crate::login_system::simulator_login_protocol::{
    SimulatorLoginOptions, SimulatorLoginProtocol,
};
use crate::packet_types::PacketType;
use std::env;
use std::error::Error;

use std::io::Cursor;
use mac_address::get_mac_address;
use md5::{Digest, Md5};
use reqwest::Client;
use std::fs::File;

extern crate sys_info;
use crate::header::{Header, PacketFrequency};
use crate::packet::{Packet, PacketData};
use reqwest::header::{CONTENT_LENGTH, CONTENT_TYPE, USER_AGENT};
use std::io::{self, BufRead, Read};

use super::errors::create_login_error_from_message;

#[derive(Debug, Clone)]
pub struct Login {
    pub first: String,
    pub last: String,
    pub passwd: String,
    pub start: String,
    pub channel: String,
    pub agree_to_tos: bool,
    pub read_critical: bool,
    pub url: String,
}
///Logs in using a SimulatorLoginProtocol object and the url string.
/// returns a LoginResult, or an error.
/// If the login response xml can successfully be converted into a LoginResponse struct, do that and
/// return the struct.
/// If it can't, try to convert it to a LoginFailure.
/// If it isn't of that format either (which would be very bad), return an error
///
///```
///let login_data = build_login(Login {
///    first: "default".to_string(),
///    last: "user".to_string(),
///    passwd: "password".to_string(),
///    start: "home".to_string(),
///    channel: "benthic".to_string(),
///    agree_to_tos: true,
///    read_critical: true,
///});
///tokio::task::spawn_blocking(|| {
///    let login_response = login(
///        login_data,
///        build_test_url("http://127.0.0.1", 9000),
///    );
///    match login_response {
///        Ok(LoginResult::Success(response)) => {
///            assert!(response.first_name == *"default");
///            assert!(response.last_name == *"user");
///        }
///        Ok(LoginResult::Failure(failure)) => {
///            println!("Login failed: {}", failure.message);
///        }
///        Err(e) => panic!("Login failed: {:?}", e),
///    }
/// });
/// ```
pub async fn login(
    login_data: SimulatorLoginProtocol,
    url: String,
) -> Result<LoginResponse, LoginError> {
    let req = xmlrpc::Request::new("login_to_simulator").arg(login_data);
    let client = Client::new();

    let mut body = Vec::new();
    let mut login_response = Vec::new();
    req.write_as_xml(&mut body).unwrap();

    let mut response = match client
        .post(url)
        .header(USER_AGENT, "benthic")
        .header(CONTENT_TYPE, "text/xml; charset=utf-8")
        .header(CONTENT_LENGTH, body.len())
        .body(body)
        .send()
        .await
    {
        Ok(response) => response,
        Err(e) => return Err(LoginError::new(Reason::Connection, &format!("1{:?}", e))),
    };

    while let Some(chunk) = response.chunk().await.map_err(|e| LoginError::new(Reason::Connection, &format!("{:?}", e)))?{
        login_response.extend_from_slice(&chunk);
    }
 
    let mut reader = Cursor::new(login_response);
    let parsed_data = match xmlrpc::parser::parse_response(&mut reader){
        Ok(data) => match data {
            Ok(data) => data,
            Err(e) => return Err(LoginError::new(Reason::Connection, &format!("{:?}", e)))
        },
        Err(e) => {return Err(LoginError::new(Reason::Connection, &format!("{:?}", e)))}
    };

    let parsed_data_clone = parsed_data.clone();
    match LoginResponse::try_from(parsed_data) {
        Ok(login_response) => Ok(login_response),
        Err(_) => Err(create_login_error_from_message(parsed_data_clone))
    }
}

///Generates a SimulatorLoginProtocol based on user supplied values
///returns a SimulatorLoginProtocol
///```
///use metaverse_login::login::{build_struct_with_defaults};
///
///let login_struct = build_login(Login{
///                         first: "default".to_string(),
///                         last: "user".to_string(),
///                         start: "home".to_string(),
///                         channel: "benthic".to_string(),
///                         agree_to_tos: true,
///                         read_critical: true
///                         });
///assert_eq!(login_struct.first, "first");
impl SimulatorLoginProtocol {
    pub fn new(login: Login) -> Self {
        SimulatorLoginProtocol {
            first: login.first,
            last: login.last,
            passwd: hash_passwd(login.passwd),
            start: login.start,
            channel: login.channel,
            version: env!("CARGO_PKG_VERSION").to_string(),
            platform: match env::consts::FAMILY {
                "mac" => "mac".to_string(),
                "win" => "win".to_string(),
                "unix" => "lin".to_string(),
                _ => "lin".to_string(),
            },
            platform_string: sys_info::os_release().unwrap_or_default(),
            platform_version: sys_info::os_release().unwrap_or_default(),
            mac: match get_mac_address() {
                Ok(Some(mac)) => format!("{}", mac),
                _ => format!("{}", 00000000000000000000000000000000),
            },
            id0: "unused".to_string(), // Provide a default value for id0. This is unused by default
            agree_to_tos: login.agree_to_tos,
            read_critical: login.read_critical,
            viewer_digest: match hash_viewer_digest() {
                Ok(viewer_digest) => Some(viewer_digest),
                Err(_) => Some("unused".to_string()),
            },
            address_size: 64,         // Set a default value if needed
            extended_errors: true,    // Set a default value if needed
            last_exec_event: None,    // Default to None
            last_exec_duration: 0,    // Set a default value if needed
            skipoptional: None,       // Default to None
            host_id: "".to_string(),  // Set a default value if needed
            mfa_hash: "".to_string(), // Set a default value if needed
            token: "".to_string(),    // Set a default value if needed
            options: SimulatorLoginOptions::default(), // Use default options
        }
    }
}

/// md5 hashes the password
fn hash_passwd(passwd_raw: String) -> String {
    let mut hasher = md5::Md5::new();
    hasher.update(passwd_raw);
    format!("$1${:x}", hasher.finalize())
}

/// Creates the viewer digest, a fingerprint of the viewer executable
/// this isn't used by opensimulator, but it's fun to have
fn hash_viewer_digest() -> Result<String, Box<dyn Error>> {
    let path = env::args().next().ok_or("No argument found")?;
    let mut f = File::open(path)?;
    let mut byt = Vec::new();
    f.read_to_end(&mut byt)?;

    let mut hasher = Md5::new();
    hasher.update(&byt);
    let hash = hasher.finalize();

    Ok(format!("{:x}", hash))
}

impl PacketData for Login {
    fn from_bytes(bytes: &[u8]) -> io::Result<Self> {
        let mut cursor = std::io::Cursor::new(bytes);

        let read_string = |cursor: &mut std::io::Cursor<&[u8]>| -> io::Result<String> {
            let mut buffer = Vec::new();
            cursor.read_until(0, &mut buffer)?;
            buffer.pop(); // Remove null terminator
            String::from_utf8(buffer).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
        };

        let first = read_string(&mut cursor)?;
        let last = read_string(&mut cursor)?;
        let passwd = read_string(&mut cursor)?;
        let start = read_string(&mut cursor)?;
        let channel = read_string(&mut cursor)?;

        let mut bool_buffer = [0u8; 2];
        cursor.read_exact(&mut bool_buffer)?;
        let agree_to_tos = bool_buffer[0] != 0;
        let read_critical = bool_buffer[1] != 0;

        let url = read_string(&mut cursor)?;

        Ok(Login {
            first,
            last,
            passwd,
            start,
            channel,
            agree_to_tos,
            read_critical,
            url,
        })
    }

    fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend(self.first.as_bytes());
        bytes.push(0);
        bytes.extend(self.last.as_bytes());
        bytes.push(0);
        bytes.extend(self.passwd.as_bytes());
        bytes.push(0);
        bytes.extend(self.start.as_bytes());
        bytes.push(0);
        bytes.extend(self.channel.as_bytes());
        bytes.push(0);
        bytes.push(self.agree_to_tos as u8);
        bytes.push(self.read_critical as u8);
        bytes.extend(self.url.as_bytes());
        bytes.push(0);
        bytes
    }
}

impl Packet {
    pub fn new_login_packet(login_packet: Login) -> Self {
        Packet {
            header: Header {
                id: 66,
                frequency: PacketFrequency::Fixed,
                reliable: true,
                sequence_number: 0,
                appended_acks: false,
                zerocoded: false,
                resent: false,
                ack_list: None,
                size: None,
            },
            body: PacketType::Login(Box::new(login_packet)),
        }
    }
}

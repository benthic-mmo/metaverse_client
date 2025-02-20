use crate::client_update_data::ClientUpdateData;
use crate::packet::MessageType;
use crate::login::errors::{create_login_error_from_message, LoginError, Reason};
use crate::login::login_response::LoginResponse;
use crate::login::simulator_login_protocol::{SimulatorLoginOptions, SimulatorLoginProtocol};
use std::env;
use std::error::Error;

use mac_address::get_mac_address;
use md5::{Digest, Md5};
use std::fs::File;

extern crate sys_info;
use crate::header::{Header, PacketFrequency};
use crate::packet::{Packet, PacketData};
use futures::future::BoxFuture;
use std::any::Any;
use std::collections::HashMap;
use std::io::{self, BufRead, Read};
use std::sync::Arc;
use std::sync::Mutex;
use tokio::sync::oneshot::Sender;


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
pub fn login(login_data: SimulatorLoginProtocol, url: String) -> Result<LoginResponse, LoginError> {
    let req = xmlrpc::Request::new("login_to_simulator").arg(login_data);
    let request = match req.call_url(url) {
        Ok(request) => request,
        Err(e) => {
            return Err(LoginError::new(
                Reason::Connection,
                &e.to_string(),
            ))
        }
    };

    if let Ok(login_response) = LoginResponse::try_from(request.clone()) {
        return Ok(login_response);
    }

    Err(create_login_error_from_message(request))
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
            Ok(String::from_utf8(buffer)
                .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?)
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

    fn on_receive(
        &self,
        _: Arc<Mutex<HashMap<u32, Sender<()>>>>,
        _: Arc<Mutex<Vec<ClientUpdateData>>>,
    ) -> BoxFuture<'static, ()> {
        Box::pin(async move {
            println!("Login packet received");
        })
    }

    fn message_type(&self) -> MessageType {
        MessageType::Login
    }

    fn as_any(&self) -> &dyn Any {
        self
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
            body: Arc::new(login_packet),
        }
    }
}

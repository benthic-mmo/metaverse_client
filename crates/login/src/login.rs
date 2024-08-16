use crate::models::errors::{create_login_error_from_message, LoginError};
use crate::models::login_response::LoginResponse;
use crate::models::simulator_login_protocol::{
    Login, SimulatorLoginOptions, SimulatorLoginProtocol,
};
use md5;
use std::env;
use std::error::Error;

use mac_address::get_mac_address;
use md5::{Digest, Md5};
use std::fs::File;
use std::io::Read;

extern crate sys_info;

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
pub fn login(
    login_data: SimulatorLoginProtocol,
    url: String,
) -> Result<LoginResponse, LoginError> {
    let req = xmlrpc::Request::new("login_to_simulator").arg(login_data);
    let request = match req.call_url(url) {
        Ok(request) => request,
        Err(e) => return Err(LoginError::new(crate::models::errors::Reason::Connection, &e.to_string())),
    };

    if let Ok(login_response) = LoginResponse::try_from(request.clone()) {
        return Ok(login_response)
    }

    return Err(create_login_error_from_message(request));
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
    pub fn new (login: Login) -> Self {
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
        address_size: 64,                          // Set a default value if needed
        extended_errors: true,                     // Set a default value if needed
        last_exec_event: None,                     // Default to None
        last_exec_duration: 0,                     // Set a default value if needed
        skipoptional: None,                        // Default to None
        host_id: "".to_string(),                   // Set a default value if needed
        mfa_hash: "".to_string(),                  // Set a default value if needed
        token: "".to_string(),                     // Set a default value if needed
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
    let hash = Md5::new().chain(&byt).finalize();
    Ok(format!("{:x}", hash))
}

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
///returns a String containing the server's status code
pub fn login(login_data: SimulatorLoginProtocol, url: String) -> xmlrpc::Value {
    let req = xmlrpc::Request::new("login_to_simulator").arg(login_data);
    req.call_url(url).unwrap()
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
pub fn build_login(login: Login) -> SimulatorLoginProtocol {
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
        id0: "unused".to_string(), // Provide a default value for id0
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
/// md5 hashes the password
fn hash_passwd(passwd_raw: String) -> String {
    let mut hasher = md5::Md5::new();
    hasher.update(passwd_raw);
    format!("$1${:x}", hasher.finalize())
}

/// Creates the viewer digest, a fingerprint of the viewer executable
fn hash_viewer_digest() -> Result<String, Box<dyn Error>> {
    let path = env::args().next().ok_or("No argument found")?;
    let mut f = File::open(path)?;
    let mut byt = Vec::new();
    f.read_to_end(&mut byt)?;
    let hash = Md5::new().chain(&byt).finalize();
    Ok(format!("{:x}", hash))
}

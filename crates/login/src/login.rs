use crate::models::simulator_login_protocol::{SimulatorLoginOptions, SimulatorLoginProtocol};
use hyper::header::CONTENT_TYPE;
use hyper::{Body, Client, Request};
use md5;
use md5::Digest;
use regex::Regex;
use std::env;
use std::error::Error;

use mac_address::get_mac_address;
use std::io::Read;

extern crate sys_info;

///Logs in using a SimulatorLoginProtocol object and the url string.
///returns an xmlrpc::Value containing the server's response
///
///# Examples
///
///this test always fails because there is no xmlrpc server running
///see tests/login.rs, test_lib_osgrid_connect, test_lib_osgrid_minimal for more information
///```
///use metaverse_login::models::simulator_login_protocol::{SimulatorLoginProtocol};
///use metaverse_login::login::{login};
///use std::panic;
///
///let url = "http://127.0.0.1:80";
///let login_data: SimulatorLoginProtocol = SimulatorLoginProtocol {
///     first: "first".to_string(),
///     last: "last".to_string(),
///     passwd: "password".to_string(),
///     start: "home".to_string(),
///     ..SimulatorLoginProtocol::default()
///};
///
///panic::catch_unwind(|| login(login_data, url.to_string()));
///```
pub async fn login(
    login_data: SimulatorLoginProtocol,
    url: String,
) -> Result<String, Box<dyn Error>> {
    let req = xmlrpc::Request::new("login_to_simulator").arg(login_data);
    let xml = match clean_xml(req) {
        Ok(xml) => xml,
        Err(e) => return Err(format!("failed to log in: {}", e).into()),
    };

    let client = Client::new();

    // Create the HTTP request
    let req = Request::builder()
        .method(hyper::Method::POST)
        .uri(url)
        .header(CONTENT_TYPE, "text/xml")
        .body(Body::from(xml))
        .expect("Failed to build request");

    // Send the request
    let res = client.request(req).await?;

    let status_code = res.status().as_u16().to_string();

    Ok(status_code)
}

pub fn clean_xml(xml: xmlrpc::Request) -> Result<String, Box<dyn Error>> {
    let mut output: Vec<u8> = vec![];
    xml.write_as_xml(&mut output)?;
    let request_string = String::from_utf8(output).map_err(|e| Box::new(e) as Box<dyn Error>)?;

    let re_i4 = Regex::new(r"<i4>").unwrap();
    let re_close_i4 = Regex::new(r"</i4>").unwrap();
    let re_i8 = Regex::new(r"<i8>").unwrap();
    let re_close_i8 = Regex::new(r"</i4>").unwrap();

    // Replace i4 and i8 with int
    let i4_open = re_i4.replace_all(&request_string, "<int>");
    let i4_closed = re_close_i4.replace_all(&i4_open, "</int>");
    let i8_open = re_i8.replace_all(&i4_closed, "<int>");
    let final_string = re_close_i8.replace_all(&i8_open, "</int>").into_owned();

    Ok(final_string)
}

///Logs in using a generated SimulatorLoginProtocol object
///returns an xmlrpc::Value containing the server's response
///
///# Examples
///
///this test always fails because there is no xmlrpc server running
///see tests/login.rs test_lib_build_struct_with_defaults for more information
///```
/// use metaverse_login::login::{login_with_defaults};
/// use std::panic;
///
/// let url = "http://127.0.0.1:80";
/// panic::catch_unwind(||login_with_defaults(
///                         "login_test".to_string(),
///                         "first".to_string(),
///                         "last".to_string(),
///                         "password".to_string(),
///                         "home".to_string(),
///                         true,
///                         true,
///                         url.to_string()));
///```
pub fn login_with_defaults(
    channel: String,
    first: String,
    last: String,
    passwd: String,
    start: String,
    agree_to_tos: bool,
    read_critical: bool,
    url: String,
) -> Result<xmlrpc::Value, Box<dyn Error>> {
    let login_data = build_struct_with_defaults(
        channel,
        first,
        last,
        passwd,
        start,
        agree_to_tos,
        read_critical,
    );
    let req = xmlrpc::Request::new("login_to_simulator").arg(login_data);
    match req.call_url(url) {
        Ok(value) => Ok(value),
        Err(e) => Err(Box::new(e)),
    }
}

///Generates a SimulatorLoginProtocol based on runtime values
///returns a SimulatorLoginProtocol
///```
///use metaverse_login::login::{build_struct_with_defaults};
///
///let login_struct = build_struct_with_defaults(
///         "channel".to_string(),
///         "first".to_string(),
///         "last".to_string(),
///         "passwd".to_string(),
///         "home".to_string(),
///         true,
///         true);
///assert_eq!(login_struct.first, "first");
///```
///
///
///
pub fn build_struct_with_defaults(
    channel: String,
    first: String,
    last: String,
    passwd: String,
    start: String,
    agree_to_tos: bool,
    read_critical: bool,
) -> SimulatorLoginProtocol {
    SimulatorLoginProtocol {
        first,
        last,
        passwd: hash_passwd(passwd),
        start,
        channel,
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
        id0: "default_id0".to_string(), // Provide a default value for id0
        agree_to_tos,
        read_critical,
        viewer_digest: Some(hash_viewer_digest()), // Assuming hash_viewer_digest() returns a String
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
    let data = format!("$1${:x}", hasher.finalize());
    data
}

/// Creates the viewer digest, a fingerprint of the viewer executable
fn hash_viewer_digest() -> String {
    let mut f = std::fs::File::open(std::env::args().next().unwrap()).unwrap();
    let mut byt = Vec::new();
    f.read_to_end(&mut byt).unwrap();
    let hash = md5::Md5::new().chain(&byt).finalize();
    let s = format!("{:x}", hash);
    s
}

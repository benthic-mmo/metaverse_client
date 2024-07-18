// integration test for login
extern crate sys_info;

use lazy_static::lazy_static;
use std::collections::HashMap;
use std::env;
use std::net::TcpStream;
use std::panic;
use std::process::{Child, Command};
use std::thread::sleep;
use std::time::{Duration, Instant};

use metaverse_login::login::{build_struct_with_defaults, login, login_with_defaults};
use metaverse_login::models::simulator_login_protocol::{
    SimulatorLoginOptions, SimulatorLoginProtocol,
};

// port and address for the test server
const PYTHON_PORT: u16 = 9000;
const PYTHON_URL: &str = "http://127.0.0.1";
const OSGRID_PORT: u16 = 80;
const OSGRID_URL: &str = "http://login.osgrid.org";

lazy_static! {
    static ref EXAMPLE_LOGIN: SimulatorLoginProtocol = SimulatorLoginProtocol {
        first: "1".to_string(),
        last: "1".to_string(),
        passwd: "1".to_string(),
        start: "1".to_string(),
        channel: Some("1".to_string()),
        version: Some("1".to_string()),
        platform: Some("1".to_string()),
        platform_string: Some("1".to_string()),
        platform_version: Some("1".to_string()),
        mac: Some("1".to_string()),
        id0: Some("1".to_string()),
        agree_to_tos: Some(false),
        read_critical: Some(false),
        viewer_digest: Some("1".to_string()),
        address_size: Some("1".to_string()),
        extended_errors: Some("1".to_string()),
        last_exec_event: Some(1),
        last_exec_duration: Some("1".to_string()),
        skipoptional: Some(false),
        options: Some(SimulatorLoginOptions {
            adult_compliant: Some("1".to_string()),
            advanced_mode: Some("1".to_string()),
            avatar_picker_url: Some("1".to_string()),
            buddy_list: Some("1".to_string()),
            classified_categories: Some("1".to_string()),
            currency: Some("1".to_string()),
            destination_guide_url: Some("1".to_string()),
            display_names: Some("1".to_string()),
            event_categories: Some("1".to_string()),
            gestures: Some("1".to_string()),
            global_textures: Some("1".to_string()),
            inventory_root: Some("1".to_string()),
            inventory_skeleton: Some("1".to_string()),
            inventory_lib_root: Some("1".to_string()),
            inventory_lib_owner: Some("1".to_string()),
            inventory_skel_lib: Some("1".to_string()),
            login_flags: Some("1".to_string()),
            max_agent_groups: Some("1".to_string()),
            max_groups: Some("1".to_string()),
            map_server_url: Some("1".to_string()),
            newuser_config: Some("1".to_string()),
            search: Some("1".to_string()),
            tutorial_setting: Some("1".to_string()),
            ui_config: Some("1".to_string()),
            voice_config: Some("1".to_string())
        })
    };
}

struct Reap(Child);
impl Drop for Reap {
    fn drop(&mut self) {
        self.0.kill().expect("process already died");
    }
}

///Tests login struct against a dummy python server
/// verifies that the xml being sent is valid and can be read by other servers
#[test]
fn test_struct_python_validate_xml() {
    let mut reaper = match setup() {
        Ok(reap) => reap,
        Err(_string) => return,
    };

    match reaper.0.try_wait().unwrap() {
        None => {}
        Some(status) => {
            panic!("python process unexpectedly exited: {}", status);
        }
    }

    let test_server_url = build_test_url(PYTHON_URL, PYTHON_PORT);
    let login_response = send_login(EXAMPLE_LOGIN.clone(), test_server_url.clone());
    assert_eq!(login_response["first_name"], xmlrpc::Value::from("None"));
    assert_eq!(login_response["last_name"], xmlrpc::Value::from("None"));
    assert_eq!(login_response["login"], xmlrpc::Value::from("None"));

    match reaper.0.try_wait().unwrap() {
        None => {}
        Some(status) => {
            panic!("python process unexpectedly exited: {}", status);
        }
    }
}

///Tests connectivity with osgrid, attempts login with invalid credentials
#[test]
fn test_struct_osgrid_connect() {
    let prod_server_url = build_test_url(OSGRID_URL, OSGRID_PORT);
    let login_response = send_login(EXAMPLE_LOGIN.clone(), prod_server_url.clone());
    assert_eq!(login_response["login"], xmlrpc::Value::from("false"));
    assert_eq!(login_response["reason"], xmlrpc::Value::from("key"));
}

///Tests login with live credentials. Creds need to be set in the .creds.toml file
///uses your real username and password so be careful not to commit this file !!
#[test]
fn test_struct_osgrid_login() {
    let creds = match read_creds() {
        Some(x) => x,
        None => {
            println!("test skipped, no creds file");
            return;
        }
    };
    let creds_firstname = creds.get("first").unwrap().to_string();
    let creds_lastname = creds.get("last").unwrap().to_string();

    let auth_login: SimulatorLoginProtocol = SimulatorLoginProtocol {
        first: creds_firstname.clone(),
        last: creds_lastname.clone(),
        passwd: creds.get("passwd").unwrap().to_string(),
        start: creds.get("start").unwrap().to_string(),
        channel: Some(creds.get("channel").unwrap().to_string()),
        version: Some(creds.get("version").unwrap().to_string()),
        platform: Some(creds.get("platform").unwrap().to_string()),
        platform_string: Some(creds.get("platform").unwrap().to_string()),
        platform_version: Some(creds.get("platform_version").unwrap().to_string()),
        mac: Some(creds.get("mac").unwrap().to_string()),
        id0: Some(creds.get("id0").unwrap().to_string()),
        agree_to_tos: Some(true),
        read_critical: Some(true),
        ..SimulatorLoginProtocol::default()
    };

    let prod_server_url = build_test_url(OSGRID_URL, OSGRID_PORT);
    let login_response = send_login(auth_login.clone(), prod_server_url.clone());

    validate_grid_response(login_response, creds_firstname, creds_lastname)
}

///Tests the smallest possible request that will facilitate a login on osgrid
#[test]
fn test_struct_osgrid_login_minimal() {
    let creds = match read_creds() {
        Some(x) => x,
        None => {
            println!("test skipped, no creds file");
            return;
        }
    };
    let creds_firstname = creds.get("first").unwrap().to_string();
    let creds_lastname = creds.get("last").unwrap().to_string();

    let auth_login: SimulatorLoginProtocol = SimulatorLoginProtocol {
        first: creds_firstname.clone(),
        last: creds_lastname.clone(),
        passwd: creds.get("passwd").unwrap().to_string(),
        start: creds.get("start").unwrap().to_string(),
        ..SimulatorLoginProtocol::default()
    };

    let prod_server_url = build_test_url(OSGRID_URL, OSGRID_PORT);
    let login_response = send_login(auth_login.clone(), prod_server_url.clone());

    validate_grid_response(login_response, creds_firstname, creds_lastname)
}

///Tests the library login function against python server
#[test]
fn test_lib_login_python() {
    let mut reaper = match setup() {
        Ok(reap) => reap,
        Err(_string) => return,
    };

    match reaper.0.try_wait().unwrap() {
        None => {}
        Some(status) => {
            panic!("python process unexpectedly exited: {}", status);
        }
    }

    let test_server_url = build_test_url(PYTHON_URL, PYTHON_PORT);
    let test_login: SimulatorLoginProtocol = SimulatorLoginProtocol {
        first: "first".to_string(),
        last: "last".to_string(),
        passwd: "passwd".to_string(),
        start: "last".to_string(),
        ..SimulatorLoginProtocol::default()
    };

    let login_response = login(test_login, test_server_url);

    assert_eq!(login_response["first_name"], xmlrpc::Value::from("None"));
    assert_eq!(login_response["last_name"], xmlrpc::Value::from("None"));
    assert_eq!(login_response["login"], xmlrpc::Value::from("None"));

    match reaper.0.try_wait().unwrap() {
        None => {}
        Some(status) => {
            panic!("python process unexpectedly exited: {}", status);
        }
    }
}

///Tests connectivity with osgrid, attempts login with invalid credentials using library
#[test]
fn test_lib_osgrid_connect() {
    let prod_server_url = build_test_url(OSGRID_URL, OSGRID_PORT);
    let login_response = login(EXAMPLE_LOGIN.clone(), prod_server_url);
    assert_eq!(login_response["login"], xmlrpc::Value::from("false"));
    assert_eq!(login_response["reason"], xmlrpc::Value::from("key"));
}

///Tests the smallest possible request that will facilitate a login on osgrid
#[test]
fn test_lib_osgrid_login_minimal() {
    let creds = match read_creds() {
        Some(x) => x,
        None => {
            println!("test skipped, no creds file");
            return;
        }
    };
    let creds_firstname = creds.get("first").unwrap().to_string();
    let creds_lastname = creds.get("last").unwrap().to_string();

    let auth_login: SimulatorLoginProtocol = SimulatorLoginProtocol {
        first: creds_firstname.clone(),
        last: creds_lastname.clone(),
        passwd: creds.get("passwd").unwrap().to_string(),
        start: creds.get("start").unwrap().to_string(),
        ..SimulatorLoginProtocol::default()
    };

    let prod_server_url = build_test_url(OSGRID_URL, OSGRID_PORT);
    let login_response = login(auth_login.clone(), prod_server_url.clone());

    validate_grid_response(login_response, creds_firstname, creds_lastname)
}

#[test]
fn test_lib_build_struct_with_defaults() {
    let default_struct = build_struct_with_defaults(
        env!("CARGO_CRATE_NAME").to_string(),
        "first".to_string(),
        "last".to_string(),
        "passwd".to_string(),
        "home".to_string(),
        true,
        true,
    );

    assert_eq!(
        default_struct.platform_string.unwrap(),
        sys_info::os_release().unwrap()
    );
    assert_eq!(
        default_struct.platform.unwrap(),
        match env::consts::FAMILY {
            "mac" => "mac".to_string(),
            "win" => "win".to_string(),
            "unix" => "lin".to_string(),
            _ => "lin".to_string(),
        }
    );
}

///Tests the default login builder
#[test]
fn test_lib_osgrid_login_defaults() {
    let creds = match read_creds() {
        Some(x) => x,
        None => {
            println!("test skipped, no creds file");
            return;
        }
    };

    let creds_firstname = creds.get("first").unwrap().to_string();
    let creds_lastname = creds.get("last").unwrap().to_string();

    let prod_server_url = build_test_url(OSGRID_URL, OSGRID_PORT);
    let login_response = login_with_defaults(
        env!("CARGO_CRATE_NAME").to_string(),
        creds_firstname.clone(),
        creds_lastname.clone(),
        creds.get("passwd").unwrap().to_string(),
        creds.get("start").unwrap().to_string(),
        true,
        true,
        prod_server_url,
    );
    validate_grid_response(login_response, creds_firstname, creds_lastname)
}

fn read_creds() -> Option<HashMap<String, String>> {
    let mut settings = config::Config::default();
    match settings.merge(config::File::with_name(".creds")) {
        Ok(_file) => _file,
        Err(..) => {
            return None;
        }
    };
    settings
        .merge(config::Environment::with_prefix("APP"))
        .unwrap();

    Some(settings.try_into::<HashMap<String, String>>().unwrap())
}

fn validate_grid_response(login_response: xmlrpc::Value, firstname: String, lastname: String) {
    println!("{:?}", login_response);
    let verify = panic::catch_unwind(|| {
        assert_eq!(login_response["login"], xmlrpc::Value::from("true"));
        assert_eq!(login_response["first_name"], xmlrpc::Value::from(firstname));
        assert_eq!(login_response["last_name"], xmlrpc::Value::from(lastname));
    });
    if verify.is_err() {
        assert_eq!(login_response["reason"], xmlrpc::Value::from("presence"))
    }
}

/// sets up python xmlrpc server for testing
fn setup() -> Result<Reap, String> {
    // logs when server started
    let start = Instant::now();
    // runs the python command to start the test server
    let mut child = match Command::new("python3")
        .arg("tests/test_server/test_server.py")
        .spawn()
    {
        Ok(child) => child,
        Err(e) => {
            eprintln!("could not start test server, ignoring test({})", e);
            return Err("Could not start test server".to_string());
        }
    };

    // logs how many tries it took to connect to server
    // attempts to connect to python server
    for iteration in 0.. {
        match child.try_wait().unwrap() {
            None => {}
            Some(status) => panic!("python process died {}", status),
        }
        match TcpStream::connect(("127.0.0.1", PYTHON_PORT)) {
            Ok(_) => {
                println!(
                    "connected to server after {:?} (iteration{})",
                    Instant::now() - start,
                    iteration
                );
                return Ok(Reap(child));
            }
            Err(_) => {}
        }
        sleep(Duration::from_millis(50));
    }
    Ok(Reap(child))
}

/// helper function for building URL. May be unnescecary
fn build_test_url(url: &str, port: u16) -> String {
    let mut url_string = "".to_owned();
    url_string.push_str(url);
    url_string.push(':');
    url_string.push_str(&port.to_string());
    println!("url string {}", url_string);
    url_string
}

/// sends login for testing struct
fn send_login(example_login: SimulatorLoginProtocol, url_string: String) -> xmlrpc::Value {
    // Login to test server
    let req = xmlrpc::Request::new("login_to_simulator").arg(example_login);
    debug_request_xml(req.clone());

    let login = req.call_url(url_string).unwrap();
    debug_response_xml(login.clone());
    println!("struct data: {:?}", login);
    login
}

/// prints out xml of request for debugging
fn debug_request_xml(xml: xmlrpc::Request) {
    let mut debug: Vec<u8> = vec![];
    match xml.write_as_xml(&mut debug) {
        Ok(_) => println!("xml request: {:?}", String::from_utf8(debug)),
        Err(e) => println!("failed to debug request xml {}", e),
    };
}

/// prints out xml of response for debugging
fn debug_response_xml(xml: xmlrpc::Value) {
    let mut debug: Vec<u8> = vec![];
    match xml.write_as_xml(&mut debug) {
        Ok(_) => println!("xml response: {:?}", String::from_utf8(debug)),
        Err(e) => println!("failed to debug response xml {}", e),
    };
}

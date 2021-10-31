// integration test for login

#[macro_use]
extern crate lazy_static;

use std::collections::HashMap;
use std::net::TcpStream;
use std::panic;
use std::process::{Child, Command};
use std::thread::sleep;
use std::time::{Duration, Instant};

use login::models::simulator_login_protocol::{SimulatorLoginOptions, SimulatorLoginProtocol};

// port and address for the test server
const PORT: u16 = 9000;
const URL: &'static str = "http://127.0.0.1";

lazy_static! {
    static ref EXAMPLE_LOGIN: SimulatorLoginProtocol = SimulatorLoginProtocol {
        first: "1".to_string(),
        last: "2".to_string(),
        passwd: "1".to_string(),
        start: "1".to_string(),
        channel: "1".to_string(),
        version: "1".to_string(),
        platform: "1".to_string(),
        platform_string: "1".to_string(),
        platform_version: "1".to_string(),
        mac: "1".to_string(),
        id0: "1".to_string(),
        agree_to_tos: false,
        read_critical: false,
        viewer_digest: "1".to_string(),
        address_size: "1".to_string(),
        extended_errors: "1".to_string(),
        last_exec_event: 1,
        last_exec_duration: "1".to_string(),
        skipoptional: false,
        options: SimulatorLoginOptions {
            adult_compliant: "1".to_string(),
            advanced_mode: "1".to_string(),
            avatar_picker_url: "1".to_string(),
            buddy_list: "1".to_string(),
            classified_categories: "1".to_string(),
            currency: "1".to_string(),
            destination_guide_url: "1".to_string(),
            display_names: "1".to_string(),
            event_categories: "1".to_string(),
            gestures: "1".to_string(),
            global_textures: "1".to_string(),
            inventory_root: "1".to_string(),
            inventory_skeleton: "1".to_string(),
            inventory_lib_root: "1".to_string(),
            inventory_lib_owner: "1".to_string(),
            inventory_skel_lib: "1".to_string(),
            login_flags: "1".to_string(),
            max_agent_groups: "1".to_string(),
            max_groups: "1".to_string(),
            map_server_url: "1".to_string(),
            newuser_config: "1".to_string(),
            search: "1".to_string(),
            tutorial_setting: "1".to_string(),
            ui_config: "1".to_string(),
            voice_config: "1".to_string()
        }
    };
}

struct Reap(Child);
impl Drop for Reap {
    fn drop(&mut self) {
        self.0.kill().expect("process already died");
    }
}

///Tests login struct against a dummy python server
#[test]
fn test_python() {
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

    let test_server_url = build_test_url(URL, PORT);
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
fn test_grid_osgrid() {
    let prod_server_url = build_test_url("http://login.osgrid.org", 80);
    let login_response = send_login(EXAMPLE_LOGIN.clone(), prod_server_url.clone());
    assert_eq!(login_response["login"], xmlrpc::Value::from("false"));
    assert_eq!(login_response["reason"], xmlrpc::Value::from("key"));
}

///Tests login with live credentials. Creds need to be set in the TestSettings.toml file
///uses your real username and password so be careful not to commit this file !!
///TODO: make this an optional test that passes or skips whel creds file is not present
#[test]
fn test_grid_osgrid_creds() {
    let mut settings = config::Config::default();
    settings
        .merge(config::File::with_name("TestSettings"))
        .unwrap()
        .merge(config::Environment::with_prefix("APP"))
        .unwrap();

    let creds = settings.try_into::<HashMap<String, String>>().unwrap();
    let creds_firstname = creds.get("first").unwrap().to_string();
    let creds_lastname = creds.get("last").unwrap().to_string();
    let auth_login: SimulatorLoginProtocol = SimulatorLoginProtocol {
        first: creds_firstname.clone(),
        last: creds_lastname.clone(),
        passwd: creds.get("passwd").unwrap().to_string(),
        start: creds.get("start").unwrap().to_string(),
        channel: creds.get("channel").unwrap().to_string(),
        version: creds.get("version").unwrap().to_string(),
        platform: creds.get("platform").unwrap().to_string(),
        platform_string: creds.get("platform").unwrap().to_string(),
        platform_version: creds.get("platform_version").unwrap().to_string(),
        mac: creds.get("mac").unwrap().to_string(),
        id0: creds.get("id0").unwrap().to_string(),
        agree_to_tos: true,
        read_critical: true,
        viewer_digest: "1".to_string(),
        address_size: "1".to_string(),
        extended_errors: "1".to_string(),
        last_exec_event: 0,
        last_exec_duration: "1".to_string(),
        skipoptional: false,
        options: SimulatorLoginOptions {
            adult_compliant: "1".to_string(),
            advanced_mode: "1".to_string(),
            avatar_picker_url: "1".to_string(),
            buddy_list: "1".to_string(),
            classified_categories: "1".to_string(),
            currency: "1".to_string(),
            destination_guide_url: "1".to_string(),
            display_names: "1".to_string(),
            event_categories: "1".to_string(),
            gestures: "1".to_string(),
            global_textures: "1".to_string(),
            inventory_root: "1".to_string(),
            inventory_skeleton: "1".to_string(),
            inventory_lib_root: "1".to_string(),
            inventory_lib_owner: "1".to_string(),
            inventory_skel_lib: "1".to_string(),
            login_flags: "1".to_string(),
            max_agent_groups: "1".to_string(),
            max_groups: "1".to_string(),
            map_server_url: "1".to_string(),
            newuser_config: "1".to_string(),
            search: "1".to_string(),
            tutorial_setting: "1".to_string(),
            ui_config: "1".to_string(),
            voice_config: "1".to_string(),
        },
    };

    let prod_server_url = build_test_url("http://login.osgrid.org", 80);
    let login_response = send_login(auth_login.clone(), prod_server_url.clone());
    let verify = panic::catch_unwind(|| {
        assert_eq!(login_response["login"], xmlrpc::Value::from("true"));
        assert_eq!(
            login_response["first_name"],
            xmlrpc::Value::from(creds_firstname)
        );
        assert_eq!(
            login_response["last_name"],
            xmlrpc::Value::from(creds_lastname)
        );
    });
    if verify.is_err() {
        assert_eq!(login_response["reason"], xmlrpc::Value::from("presence"))
    }
}

// sets up python xmlrpc server for testing
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
        match TcpStream::connect(("localhost", PORT)) {
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
    return Ok(Reap(child));
}
fn build_test_url(url: &str, port: u16) -> String {
    let mut url_string = "".to_owned();
    url_string.push_str(url);
    url_string.push_str(":");
    url_string.push_str(&port.to_string());
    println!("url string {}", url_string);
    return url_string;
}

fn send_login(example_login: SimulatorLoginProtocol, url_string: String) -> xmlrpc::Value {
    // Login to test server
    let req = xmlrpc::Request::new("login_to_simulator").arg(example_login);
    debug_request_xml(req.clone());

    let login = req.call_url(&url_string).unwrap();
    debug_response_xml(login.clone());
    println!("struct data: {:?}", login);
    return login;
}

// prints out xml of request for debugging
fn debug_request_xml(xml: xmlrpc::Request) {
    let mut debug: Vec<u8> = vec![];
    match xml.write_as_xml(&mut debug) {
        Ok(_) => println!("xml request: {:?}", String::from_utf8(debug)),
        Err(e) => println!("failed to debug request xml {}", e),
    };
}

// prints out xml of response for debugging
fn debug_response_xml(xml: xmlrpc::Value) {
    let mut debug: Vec<u8> = vec![];
    match xml.write_as_xml(&mut debug) {
        Ok(_) => println!("xml response: {:?}", String::from_utf8(debug)),
        Err(e) => println!("failed to debug response xml {}", e),
    };
}

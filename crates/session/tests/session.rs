use metaverse_login::login::{build_struct_with_defaults, login_with_defaults};
use metaverse_session::models::errors::Reason;
use metaverse_session::models::session_data::AgentAccess;
use metaverse_session::session::{connect, new_session};

use std::collections::HashMap;
use std::net::TcpStream;
use std::panic;
use std::process::{Child, Command};
use std::thread::sleep;
use std::time::{Duration, Instant};
use uuid::Uuid;

const PYTHON_PORT: u16 = 9000;
const PYTHON_URL: &'static str = "http://127.0.0.1";
const OSGRID_PORT: u16 = 80;
const OSGRID_URL: &'static str = "http://login.osgrid.org";
const THIRDROCK_PORT: u16 = 8002;
const THIRDROCK_URL: &'static str = "http://grid.3rdrockgrid.com";

struct Reap(Child);
impl Drop for Reap {
    fn drop(&mut self) {
        self.0.kill().expect("process already died");
    }
}

#[test]
fn test_mock_session() {
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

    let login_response = login_with_defaults(
        env!("CARGO_CRATE_NAME").to_string(),
        "first".to_string(),
        "last".to_string(),
        "password".to_string(),
        "last".to_string(),
        true,
        true,
        build_test_url(PYTHON_URL, PYTHON_PORT),
    );

    let session = new_session(login_response).unwrap();
    assert_eq!(
        session.home.unwrap().region_handle,
        ("r0".to_string(), "r0".to_string())
    );
    assert_eq!(
        session.look_at,
        Some(("r0".to_string(), "r0".to_string(), "r0".to_string()))
    );
    assert_eq!(session.agent_access, Some(AgentAccess::Mature));
    assert_eq!(session.agent_access_max, Some(AgentAccess::Adult));
    assert_eq!(
        session.seed_capability,
        Some("http://192.168.1.2:9000".to_string())
    );
    assert_eq!(session.first_name, Some("First".to_string()));
    assert_eq!(session.last_name, Some("Last".to_string()));
    assert_eq!(
        session.agent_id,
        Some(Uuid::parse_str("11111111-1111-0000-0000-000100bba000").unwrap())
    );
    assert_eq!(session.sim_ip, Some("192.168.1.2".to_string()));
    assert_eq!(session.sim_port, Some(9000));
    assert_eq!(session.http_port, Some(0));
    assert_eq!(session.start_location, Some("last".to_string()));
    assert_eq!(session.region_x, Some(256000));
    assert_eq!(session.region_y, Some(256000));
    assert_eq!(session.circuit_code, Some(697482820));
    assert_eq!(
        session.session_id,
        Some(Uuid::parse_str(&"6ac2e761-f490-4122-bf6c-7ad8fbb17002").unwrap())
    );
    assert_eq!(
        session.secure_session_id,
        Some(Uuid::parse_str(&"fe210274-9056-467a-aff7-d95f60bacccc".to_string()).unwrap())
    );
    assert_eq!(
        session.inventory_root.unwrap()[0].folder_id,
        "37c4cfe3-ea39-4ef7-bda3-bee73bd46d95".to_string()
    );
    let inv_skel = session.inventory_skeleton.unwrap();
    assert_eq!(inv_skel.len(), 2);
    assert_eq!(
        inv_skel[0].folder_id,
        "004d663b-9980-46ae-8559-bb60e9d67d28".to_string()
    );
    assert_eq!(
        session.inventory_lib_root.unwrap()[0].folder_id,
        "37c4cfe3-ea39-4ef7-bda3-bee73bd46d95".to_string()
    );
    let inv_skel_lib = session.inventory_skeleton_lib.unwrap();
    assert_eq!(inv_skel_lib.len(), 2);
    assert_eq!(
        inv_skel_lib[0].folder_id,
        "004d663b-9980-46ae-8559-bb60e9d67d28".to_string()
    );
    assert_eq!(
        session.inventory_lib_owner.unwrap()[0].agent_id,
        ("11111111-1111-0000-0000-000100bba000").to_string()
    );
    assert_eq!(
        session.map_server_url,
        Some("http://192.168.1.2:8002/".to_string())
    );

    let buddy_list = session.buddy_list.unwrap();
    assert_eq!(buddy_list.len(), 3);
    assert_eq!(
        buddy_list[0].buddy_id,
        "04c259b7-94bc-4822-b099-745191ffc247".to_string()
    );
    assert_eq!(buddy_list[0].buddy_rights_given.can_see_online, true);

    let gesture_list = session.gestures.unwrap();
    assert_eq!(gesture_list.len(), 2);
    assert_eq!(
        gesture_list[0].item_id,
        "004d663b-9980-46ae-8559-bb60e9d67d28".to_string()
    );
    assert_eq!(
        gesture_list[0].asset_id,
        "004d663b-9980-46ae-8559-bb60e9d67d28".to_string()
    );
    assert_eq!(
        session.initial_outfit.unwrap()[0].folder_name,
        "Nightclub Female".to_string()
    );
    assert_eq!(
        session.global_textures.unwrap()[0].sun_texture_id,
        "cce0f112-878f-4586-a2e2-a8f104bba271".to_string()
    );
    assert_eq!(session.login.unwrap(), true);
    assert_eq!(
        session.login_flags.unwrap()[0].seconds_since_epoch,
        Some(1411075065)
    );
    assert_eq!(session.message.unwrap(), "Welcome, Avatar!".to_string());
    assert_eq!(session.ui_config.unwrap()[0].allow_first_life, true);
    assert_eq!(
        session.classified_categories.unwrap()[0].category_name,
        "Shopping".to_string()
    );

    match reaper.0.try_wait().unwrap() {
        None => {}
        Some(status) => {
            panic!("python process unexpectedly exited: {}", status);
        }
    }
}

#[test]
fn test_lib_channel() {
    let defaults_struct = build_struct_with_defaults(
        env!("CARGO_CRATE_NAME").to_string(),
        "first".to_string(),
        "last".to_string(),
        "passwd".to_string(),
        "start".to_string(),
        true,
        true,
    );

    assert_eq!(
        defaults_struct.channel.unwrap(),
        env!("CARGO_CRATE_NAME").to_string()
    );
}

#[test]
fn test_lib_fail_auth_osgrid() {
    let creds = match read_creds() {
        Some(x) => x,
        None => {
            println!("test skipped, no creds file");
            return;
        }
    };

    let login_response = login_with_defaults(
        env!("CARGO_CRATE_NAME").to_string(),
        creds.get("osgridfirst").unwrap().to_string(),
        creds.get("osgridlast").unwrap().to_string(),
        "incorrectpass".to_string(),
        creds.get("osgridstart").unwrap().to_string(),
        true,
        true,
        build_test_url(OSGRID_URL, OSGRID_PORT),
    );
    let session = new_session(login_response);
    match session {
        Ok(v) => assert_eq!(v.login, Some(true)),
        Err(e) => {
            assert_eq!(e.reason, Reason::Key)
        }
    }
}

#[test]
fn test_lib_auth_osgrid() {
    let creds = match read_creds() {
        Some(x) => x,
        None => {
            println!("test skipped, no creds file");
            return;
        }
    };

    let login_response = login_with_defaults(
        env!("CARGO_CRATE_NAME").to_string(),
        creds.get("osgridfirst").unwrap().to_string(),
        creds.get("osgridlast").unwrap().to_string(),
        creds.get("osgridpasswd").unwrap().to_string(),
        creds.get("osgridstart").unwrap().to_string(),
        true,
        true,
        build_test_url(OSGRID_URL, OSGRID_PORT),
    );
    let session = new_session(login_response);
    match session {
        Ok(v) => assert_eq!(v.login, Some(true)),
        Err(e) => {
            assert_eq!(e.reason, Reason::Presence)
        }
    }
}

#[test]
fn test_lib_auth_3rdrock() {
    let creds = match read_creds() {
        Some(x) => x,
        None => {
            println!("test skipped, no creds file");
            return;
        }
    };

    let login_response = login_with_defaults(
        env!("CARGO_CRATE_NAME").to_string(),
        creds.get("3rdrockfirst").unwrap().to_string(),
        creds.get("3rdrocklast").unwrap().to_string(),
        creds.get("3rdrockpasswd").unwrap().to_string(),
        creds.get("3rdrockstart").unwrap().to_string(),
        true,
        true,
        build_test_url(THIRDROCK_URL, THIRDROCK_PORT),
    );
    let session = new_session(login_response);
    match session {
        Ok(v) => assert_eq!(v.login, Some(true)),
        Err(e) => {
            assert_eq!(e.reason, Reason::Presence)
        }
    }
}

#[test]
fn establish_conn() {
    let creds = match read_creds() {
        Some(x) => x,
        None => {
            println!("test skipped, no creds file");
            return;
        }
    };

    let login_response = login_with_defaults(
        env!("CARGO_CRATE_NAME").to_string(),
        creds.get("osgridfirst").unwrap().to_string(),
        creds.get("osgridlast").unwrap().to_string(),
        creds.get("osgridpasswd").unwrap().to_string(),
        creds.get("osgridstart").unwrap().to_string(),
        true,
        true,
        build_test_url(OSGRID_URL, OSGRID_PORT),
    );
    let session = new_session(login_response);
    tokio_test::block_on(connect(session.unwrap())).unwrap();
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

/// helper function for building URL. May be unnescecary
fn build_test_url(url: &str, port: u16) -> String {
    let mut url_string = "".to_owned();
    url_string.push_str(url);
    url_string.push_str(":");
    url_string.push_str(&port.to_string());
    println!("url string {}", url_string);
    return url_string;
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
        match TcpStream::connect(("localhost", PYTHON_PORT)) {
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

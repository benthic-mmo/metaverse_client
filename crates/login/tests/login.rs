// integration test for login
extern crate sys_info;

use log::{info, LevelFilter};
use metaverse_login::login::*;
use metaverse_login::models::login_response::LoginResult;
use metaverse_login::models::simulator_login_protocol::*;
use std::panic;
use tokio::sync::mpsc;
use tokio::time::{sleep, Duration};

use actix::Actor;
use lazy_static::lazy_static;
use metaverse_instantiator::config_generator::*;
use metaverse_instantiator::models::server::*;
use metaverse_instantiator::server::*;
use std::sync::{Arc, Mutex};
use tokio::sync::Notify;

fn init_logger() {
    let _ = env_logger::builder()
        .filter(None, LevelFilter::Info)
        .is_test(true)
        .try_init();
}

lazy_static! {
    static ref EXAMPLE_LOGIN: SimulatorLoginProtocol = SimulatorLoginProtocol {
        address_size: 64,
        agree_to_tos: false,
        channel: "benthic".to_string(),
        extended_errors: true,
        host_id: "".to_string(),
        id0: "86eb9930e48f487de8ae3e84dac73339".to_string(),
        last_exec_duration: 0,
        last_exec_event: Some(0),
        mac: "00000000000000000000000000000000".to_string(),
        mfa_hash: "".to_string(),
        passwd: "$1$5f4dcc3b5aa765d61d8327deb882cf99".to_string(),
        platform: "lnx".to_string(),
        platform_string: "Linux 6.9".to_string(),
        platform_version: "2.39.0".to_string(),
        read_critical: false,
        token: "".to_string(),
        version: "7.1.9.74745".to_string(),
        first: "default".to_string(),
        last: "user".to_string(),
        start: "home".to_string(),
        viewer_digest: None,
        skipoptional: None,
        options: SimulatorLoginOptions {
            inventory_root: Some(true),
            inventory_skeleton: Some(true),
            inventory_lib_root: Some(true),
            inventory_lib_owner: Some(true),
            inventory_skel_lib: Some(true),
            gestures: Some(true),
            display_names: Some(true),
            event_categories: Some(true),
            classified_categories: Some(true),
            adult_compliant: Some(true),
            buddy_list: Some(true),
            newuser_config: Some(true),
            ui_config: Some(true),
            advanced_mode: Some(true),
            max_agent_groups: Some(true),
            map_server_url: Some(true),
            voice_config: Some(true),
            tutorial_setting: Some(true),
            login_flags: Some(true),
            global_textures: Some(true),
            currency: Some(true),
            max_groups: Some(true),
            search: Some(true),
            destination_guide_url: Some(true),
            avatar_picker_url: Some(true),
        }
    };
}

// using metaverse_instantiator, launches a local sim server, and tests login against that.
#[actix_rt::test]
async fn test_against_local() {
    init_logger();

    let notify = Arc::new(Notify::new());
    let state = Arc::new(Mutex::new(ServerState::Starting));

    // start the sim server, and initialize logging
    let sim_server = setup_server(Arc::clone(&notify), Arc::clone(&state)).await;
    send_setup_commands(&sim_server);

    notify.notified().await;
    if *state.lock().unwrap() == ServerState::Running {
        info!("Server started. Running test commands");
        sim_server.do_send(CommandMessage{
            command: "create user default user password email@email.com 9dc18bb1-044f-4c68-906b-2cb608b2e197 default".to_string()
        });

        tokio::task::spawn_blocking(|| {
            let login_response = login(
                EXAMPLE_LOGIN.clone(),
                build_test_url("http://127.0.0.1", 9000),
            );
            match login_response {
                Ok(LoginResult::Success(response)) => {
                    assert!(response.first_name == *"default");
                    assert!(response.last_name == *"user");
                }
                Ok(LoginResult::Failure(failure)) => {
                    println!("Login failed: {}", failure.message);
                }
                Err(e) => panic!("Login failed: {:?}", e),
            }
        });
        sleep(Duration::from_secs(5)).await;
        sim_server.do_send(CommandMessage {
            command: "quit".to_string(),
        });
    } else {
        panic!("server failed to start")
    }

    notify.notified().await;
    // wait for the second notify signal to say that the server is done
}

// uses the build_login function to login with only the required user data
#[actix_rt::test]
async fn test_build_login() {
    init_logger();

    let notify = Arc::new(Notify::new());
    let state = Arc::new(Mutex::new(ServerState::Starting));

    // start the sim server, and initialize logging
    let sim_server = setup_server(Arc::clone(&notify), Arc::clone(&state)).await;
    send_setup_commands(&sim_server);

    notify.notified().await;
    if *state.lock().unwrap() == ServerState::Running {
        info!("Server started. Running test commands");
        sim_server.do_send(CommandMessage{
            command: "create user default user password email@email.com 9dc18bb1-044f-4c68-906b-2cb608b2e197 default".to_string()
        });

        let login_data = build_login(Login {
            first: "default".to_string(),
            last: "user".to_string(),
            passwd: "password".to_string(),
            start: "home".to_string(),
            channel: "benthic".to_string(),
            agree_to_tos: true,
            read_critical: true,
        });

        tokio::task::spawn_blocking(|| {
            let login_response = login(login_data, build_test_url("http://127.0.0.1", 9000));
            match login_response {
                Ok(LoginResult::Success(response)) => {
                    assert!(response.first_name == *"default");
                    assert!(response.last_name == *"user");
                }
                Ok(LoginResult::Failure(failure)) => {
                    info!("Login failed with: {}", failure.message);
                }
                Err(e) => panic!("Login failed: {:?}", e),
            }
        });

        sleep(Duration::from_secs(5)).await;
        sim_server.do_send(CommandMessage {
            command: "quit".to_string(),
        });
    } else {
        panic!("server failed to start")
    }

    notify.notified().await;
    // wait for the second notify signal to say that the server is done
}

fn send_setup_commands(sim_server: &actix::Addr<SimServer>) {
    // This is required for first time startup. This assigns the default user as the region owner.
    // TODO: make this into a sql query that automatically adds this to the default region on
    // startup
    sim_server.do_send(CommandMessage {
        command: "default".to_string(),
    });
    sim_server.do_send(CommandMessage {
        command: "user".to_string(),
    });
    sim_server.do_send(CommandMessage {
        command: "password".to_string(),
    });
    sim_server.do_send(CommandMessage {
        command: "email@email.com".to_string(),
    });
    sim_server.do_send(CommandMessage {
        command: "9dc18bb1-044f-4c68-906b-2cb608b2e197".to_string(),
    });
}

async fn setup_server(
    notify: Arc<Notify>,
    state: Arc<Mutex<ServerState>>,
) -> actix::Addr<SimServer> {
    let (stdin_sender, stdin_receiver) = mpsc::channel::<CommandMessage>(100);
    let (stdout_sender, mut receiver) = mpsc::channel::<StdoutMessage>(100);

    let (url, archive, base_dir, executable) = match read_config() {
        Ok((url, archive, base_dir, executable)) => (url, archive, base_dir, executable),
        Err(e) => panic!("Error: {}", e),
    };

    info!("downloading server. On first run, this may take a while");
    match download_sim(&url, &archive, &base_dir).await {
        Ok(_) => info!("downloaded sim successfully"),
        Err(e) => info!("failed to download sim {}", e),
    };

    let sim_server = SimServer {
        state: Arc::clone(&state),
        sim_config: create_default_config(),
        standalone_config: create_default_standalone_config(),
        regions_config: create_default_region_config(),
        process: None,
        process_stdout_sender: Some(stdout_sender),
        process_stdin_receiver: Some(stdin_receiver),
        process_stdin_sender: Some(stdin_sender),
        notify: Arc::clone(&notify),
        exec_data: ExecData {
            base_dir,
            executable,
            init_command: "mono".to_string(),
        },
    }
    .start();
    info!("Waiting for the server to start...");

    tokio::spawn(async move {
        loop {
            if let Some(msg) = receiver.recv().await {
                info!("Received message: {}", msg.log_content);
                if msg.log_content.contains("Currently selected region is") {}
            }
        }
    });
    sim_server
}

#[test]
fn test_xml_generation() {
    let req = xmlrpc::Request::new("login_to_simulator").arg(EXAMPLE_LOGIN.clone());
    debug_request_xml(req)
}

/// helper function for building URL. May be unnescecary
fn build_test_url(url: &str, port: u16) -> String {
    let mut url_string = "".to_owned();
    url_string.push_str(url);
    url_string.push(':');
    url_string.push_str(&port.to_string());
    info!("url string {}", url_string);
    url_string
}

/// prints out xml of request for debugging
fn debug_request_xml(xml: xmlrpc::Request) {
    let mut debug: Vec<u8> = vec![];
    match xml.write_as_xml(&mut debug) {
        Ok(_) => println!("xml request: {:?}", String::from_utf8(debug)),
        Err(e) => println!("failed to debug request xml {}", e),
    };
}

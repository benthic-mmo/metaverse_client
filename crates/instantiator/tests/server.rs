use log::info;
use serial_test::serial;
use tokio::sync::mpsc;
use tokio::time::{Duration, Instant};

use actix::Actor;
use metaverse_instantiator::config_generator::{
    create_default_config, create_default_region_config, create_default_standalone_config,
};
use metaverse_instantiator::models::server::*;
use metaverse_instantiator::server::*;
use std::sync::{Arc, Mutex};
use tokio::sync::Notify;

fn init_logger() {
    let _ = env_logger::builder().is_test(true).try_init();
}

#[test]
fn test_default_config() {
    let config = create_default_config();
    println!("{}", config);
}

#[test]
fn test_default_standalone_config() {
    let config = create_default_standalone_config();
    println!("{}", config);
}

#[test]
fn test_default_region_config(){
    let config = create_default_region_config();
    println!("{}", config);
}

#[actix_rt::test]
#[serial]
async fn test_sim_download() {
    init_logger();
    match read_sim_config() {
        Some(x) => x,
        None => {
            println!("test skipped, no config file");
            return;
        }
    };
    let (base_dir, _sim_executable, url, archive) = match read_config() {
            Ok((base_dir, sim_executable, url, archive)) => {
            (base_dir, sim_executable, url, archive)
        },
            Err(e) => panic!("Error: {}", e)
        };
    match download_sim(&url, &archive, &base_dir).await {
        Ok(_) => info!("downloaded sim successfully"),
        Err(e) => info!("failed to download sim {}", e)
    };
}

#[actix_rt::test]
#[serial]
async fn test_start_server() {
    init_logger();
    let (stdin_sender, stdin_receiver) = mpsc::channel::<CommandMessage>(100);

    let (url, archive, base_dir, executable) = match read_config() {
            Ok((url, archive, base_dir, executable)) => {
            (url, archive, base_dir, executable)
        },
            Err(e) => panic!("Error: {}", e)
        };
   
    match download_sim(&url, &archive, &base_dir).await {
        Ok(_) => info!("downloaded sim successfully"),
        Err(e) => info!("failed to download sim {}", e)
    };
        
    let notify = Arc::new(Notify::new());
    let state = Arc::new(Mutex::new(ServerState::Starting));

    let sim_server = SimServer {
        state: Arc::clone(&state),
        sim_config: create_default_config(),
        standalone_config: create_default_standalone_config(),
        regions_config: create_default_region_config(),
        process: None,
        process_stdout_sender: None,
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

    // Wait for the notify signal
    notify.notified().await;

    if *state.lock().unwrap() == ServerState::Running {
        info!("Server started. Running test commands");
        sim_server.do_send(CommandMessage{
            command: "create user default user password email@mail.com 9dc18bb1-044f-4c68-906b-2cb608b2e197 default".to_string()
        });

        sim_server.do_send(CommandMessage {
            command: "quit".to_string(),
        });
    } else {
        panic!("server failed to start")
    }

    // wait for the second notify signal to say that the server is done
    notify.notified().await;
}
#[actix_rt::test]
#[serial]
async fn test_stdout_capture() {
    init_logger();
    let (stdin_sender, stdin_receiver) = mpsc::channel::<CommandMessage>(100);
    let (stdout_sender, mut receiver) = mpsc::channel::<StdoutMessage>(100);

    let (url, archive, base_dir, executable) = match read_config() {
            Ok((url, archive, base_dir, executable)) => {
            (url, archive, base_dir, executable)
        },
            Err(e) => panic!("Error: {}", e)
        };
   
    match download_sim(&url, &archive, &base_dir).await {
        Ok(_) => info!("downloaded sim successfully"),
        Err(e) => info!("failed to download sim {}", e)
    };

    let notify = Arc::new(Notify::new());
    let state = Arc::new(Mutex::new(ServerState::Starting));

    SimServer {
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

    let start_time = Instant::now();
    let duration = Duration::from_secs(10);
    while start_time.elapsed() < duration {
        match receiver.recv().await {
            Some(msg) => {
                println!("Received message: {}", msg.log_content);
                if msg.log_content.contains("Currently selected region is") {
                    break;
                }
            }
            None => {
                break;
            }
        }
    }
}


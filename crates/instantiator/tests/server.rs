use log::info;
use std::fs;
use tokio::sync::mpsc;
use tokio::time::{Duration, Instant};

use actix::Actor;
use env_logger::Env;
use metaverse_instantiator::config_generator::{
    create_default_config, create_default_region_config, create_default_standalone_config,
    create_full_config,
};
use metaverse_instantiator::models::server::*;
use metaverse_instantiator::server::*;
use std::sync::{Arc, Mutex};
use tokio::sync::Notify;

#[test]
fn test_default_config() {
    let config = create_default_config();
    println!("{}", config.to_string());
}

#[test]
fn test_full_config() {
    let config = create_full_config();
    println!("{}", config.to_string());
}

#[test]
fn test_default_standalone_config() {
    let config = create_default_standalone_config();
    println!("{}", config.to_string());
}

#[actix_rt::test]
async fn test_start_server() {
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();
    let (stdin_sender, stdin_receiver) = mpsc::channel::<CommandMessage>(100);

    let conf = match read_sim_config() {
        Some(x) => x,
        None => {
            println!("test skipped, no config file");
            return;
        }
    };
    let url = conf.get("sim_url").unwrap().to_string();
    let sim_archive = conf.get("sim_archive").unwrap().to_string();
    let sim_path = conf.get("sim_path").unwrap().to_string();
    let sim_executable = conf.get("sim_executable").unwrap().to_string();

    assert!(download_sim(&url, &sim_archive, &sim_path).is_ok());

    let mut base_dir = "".to_string();
    if let Ok(canonical_path) = fs::canonicalize(sim_path) {
        base_dir = canonical_path
            .into_os_string()
            .into_string()
            .unwrap()
            .to_string();
    } else {
        assert!(false);
    }

    let notify = Arc::new(Notify::new());
    let arc_state = Arc::new(Mutex::new(ServerState::Starting));

    let sim_server = SimServer {
        state: ServerState::Starting,
        arc_state: Arc::clone(&arc_state),
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
            sim_executable,
            init_command: "mono".to_string(),
        },
    }
    .start();
    info!("Waiting for the server to start...");

    // Wait for the notify signal
    notify.notified().await;

    if *arc_state.lock().unwrap() == ServerState::Running {
        info!("Server started. Running test commands");
        sim_server.do_send(CommandMessage{
            command: "create user default user password email@mail.com 9dc18bb1-044f-4c68-906b-2cb608b2e197 default".to_string()
        });

        sim_server.do_send(CommandMessage {
            command: "quit".to_string(),
        });
    } else {
        assert!(false, "server failed to start")
    }

    // wait for the second notify signal to say that the server is done
    notify.notified().await;
}
#[actix_rt::test]
async fn test_stdout_capture() {
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();
    let (stdin_sender, stdin_receiver) = mpsc::channel::<CommandMessage>(100);
    let (stdout_sender, mut receiver) = mpsc::channel::<StdoutMessage>(100);

    let conf = match read_sim_config() {
        Some(x) => x,
        None => {
            println!("test skipped, no config file");
            return;
        }
    };
    let url = conf.get("sim_url").unwrap().to_string();
    let sim_archive = conf.get("sim_archive").unwrap().to_string();
    let sim_path = conf.get("sim_path").unwrap().to_string();
    let sim_executable = conf.get("sim_executable").unwrap().to_string();

    assert!(download_sim(&url, &sim_archive, &sim_path).is_ok());

    let mut base_dir = "".to_string();
    if let Ok(canonical_path) = fs::canonicalize(sim_path) {
        base_dir = canonical_path
            .into_os_string()
            .into_string()
            .unwrap()
            .to_string();
    } else {
        assert!(false);
    }

    let notify = Arc::new(Notify::new());
    let arc_state = Arc::new(Mutex::new(ServerState::Starting));

    SimServer {
        state: ServerState::Starting,
        arc_state: Arc::clone(&arc_state),
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
            sim_executable,
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
                    assert!(true, "output read to server ready");
                    break;
                }
            }
            None => {
                assert!(false, "Channel closed unexpectedly");
                break;
            }
        }
    }
}

#[test]
fn test_sim_download() {
    let conf = match read_sim_config() {
        Some(x) => x,
        None => {
            println!("test skipped, no config file");
            return;
        }
    };
    let url = conf.get("sim_url").unwrap().to_string();
    let sim_archive = conf.get("sim_archive").unwrap().to_string();
    let sim_path = conf.get("sim_path").unwrap().to_string();

    assert!(download_sim(&url, &sim_archive, &sim_path).is_ok())
}

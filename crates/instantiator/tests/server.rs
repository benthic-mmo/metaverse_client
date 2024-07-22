use log::info;
use std::fs;
use tokio::sync::mpsc;

use actix::Actor;
use env_logger::Env;
use metaverse_instantiator::config_generator::{
    create_default_config, create_default_region_config, create_default_standalone_config,
    create_full_config,
};
use metaverse_instantiator::models::server::*;
use metaverse_instantiator::server::*;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Notify;
use tokio::time::sleep;

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

    let sim_server = SimServer {
        state: ServerState::Stopped,
        sim_config: create_default_config(),
        standalone_config: create_default_standalone_config(),
        regions_config: create_default_region_config(),
        process: None,
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

    info!("running server commands");
    sim_server.do_send(CommandMessage{
        command: "create user default user password email@mail.com 9dc18bb1-044f-4c68-906b-2cb608b2e197 default".to_string()
    });

    sim_server.do_send(CommandMessage {
        command: "quit".to_string(),
    });

    sleep(Duration::from_secs(10)).await;
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

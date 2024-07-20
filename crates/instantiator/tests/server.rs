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
use std::time::Duration;
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

    let sim_server = SimServer {
        sim_config: create_default_config(),
        standalone_config: create_default_standalone_config(),
        regions_config: create_default_region_config(),
        process: None,
        process_stdout_sender: Some(stdout_sender)
    };

    let start_command = StartServer {
        base_dir,
        sim_executable,
        init_command: "mono".to_string(),
    };

    let sim_addr = sim_server.start();
    sim_addr.do_send(start_command);
   
    let mut received_count = 0;
    while received_count < 10 {
        match receiver.recv().await {
            Some(msg) => {
                println!("Received message: {}", msg.0);
                received_count += 1;
            }
            None => {
                println!("Channel closed unexpectedly.");
                break;
            }
        }
    }
    sleep(Duration::from_secs(5)).await;
    
    // sleep for five seconds so the setup can complete
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

#[test]
fn test_default_sim_instance() {
    let conf = match read_sim_config() {
        Some(x) => x,
        None => {
            println!("test skipped, no config file");
            return;
        }
    };
    match start_default_sim_instance(&conf) {
        Ok(rx) => {
            // Example: Continuously print output lines received from mono program
            for line in rx.iter() {
                println!("Mono program output: {}", line);
            }
        }
        Err(e) => {
            eprintln!("{}", e);
            // Handle error
        }
    }
}

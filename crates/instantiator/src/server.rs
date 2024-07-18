use env_logger::Env;
use log::{error, info, warn};
use reqwest::Url;
use std::collections::HashMap;
use std::fs;
use std::fs::File;
use std::io::{copy, BufRead, BufReader, Cursor, Read, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::mpsc::{channel, Receiver};
use std::thread;

use crate::config_generator;

pub fn read_sim_config() -> Option<HashMap<String, String>> {
    let mut settings = config::Config::default();
    match settings.merge(config::File::with_name("sim_config")) {
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

pub fn download_sim(
    url: &str,
    sim_archive: &str,
    sim_path: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    if !Path::new(sim_archive).exists() {
        let output_dir = Path::new(sim_archive).parent().unwrap();
        std::fs::create_dir_all(output_dir)?;

        info!("downloading sim {:?}", url);
        let url = Url::parse(url)?;
        let client = reqwest::blocking::Client::new();
        let mut response = client.get(url).send()?;
        let mut dest = File::create(sim_archive)?;
        copy(&mut response, &mut dest)?;
        info!("sim downloaded and saved to {}", sim_path);
    } else {
        warn!("archive already exists at {}", sim_archive);
    }

    if !Path::new(sim_path).exists() {
        let mut file = File::open(sim_archive)?;
        let mut archive = Vec::new();
        file.read_to_end(&mut archive)?;

        let target_dir = PathBuf::from(sim_path);

        zip_extract::extract(Cursor::new(archive), &target_dir, true)?;
    } else {
        warn!("sim already exists at {}", sim_path)
    }

    Ok(())
}

fn generate_config_ini(config_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    if !Path::new(&config_path).exists() {
        let default_config = config_generator::create_default_config();

        let path = Path::new(&config_path);
        let mut file = File::create(path)?;

        file.write_all(default_config.to_string().as_bytes())?;
        info!("Successfully wrote to {}", path.display());
    } else {
        warn!(
            "{} already exists. Continuing without generating defaults.",
            config_path
        );
    }
    Ok(())
}

fn generate_standalone_config_ini(config_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    if !Path::new(&config_path).exists() {
        let default_config = config_generator::create_default_standalone_config();

        let path = Path::new(&config_path);
        let mut file = File::create(path)?;

        file.write_all(default_config.to_string().as_bytes())?;
        info!("Successfully wrote to {}", path.display());
    } else {
        warn!(
            "{} already exists. Continuing without generating defaults.",
            config_path
        );
    }
    Ok(())
}
fn generate_region_config_ini(config_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    if !Path::new(&config_path).exists() {
        let default_config = config_generator::create_default_region_config();

        let path = Path::new(&config_path);
        let mut file = File::create(path)?;

        file.write_all(default_config.to_string().as_bytes())?;
        info!("Successfully wrote to {}", path.display());
    } else {
        warn!(
            "{} already exists. Continuing without generating defaults.",
            config_path
        );
    }
    Ok(())
}
pub fn start_default_sim_instance(
    conf: &HashMap<String, String>,
) -> Result<Receiver<String>, String> {
    env_logger::Builder::from_env(Env::default().default_filter_or("warn")).init();

    let url = conf.get("sim_url").unwrap().to_string();
    let sim_archive = conf.get("sim_archive").unwrap().to_string();
    let sim_path = conf.get("sim_path").unwrap().to_string();
    let sim_executable = conf.get("sim_executable").unwrap().to_string();

    let base_dir: String;
    if let Ok(canonical_path) = fs::canonicalize(&sim_path) {
        base_dir = canonical_path
            .into_os_string()
            .into_string()
            .unwrap()
            .to_string();
    } else {
        return Err("Could not start instance".to_string());
    }

    format!("{:?}", fs::canonicalize(&sim_path));

    match download_sim(&url, &sim_archive, &sim_path) {
        Ok(()) => info!("Download successful!"),
        Err(e) => error!("error downloading simulator: {}", e),
    }

    let config_path = format!("{}/bin/OpenSim.ini", base_dir);
    match generate_config_ini(&config_path) {
        Ok(file) => file,
        Err(error) => panic!("Problem opening the file: {error:?}"),
    };

    let config_path = format!("{}/bin/config-include/StandaloneCommon.ini", base_dir);
    match generate_standalone_config_ini(&config_path) {
        Ok(file) => file,
        Err(error) => panic!("Problem opening the file: {error:?}"),
    };

    let config_path = format!("{}/bin/Regions/Regions.ini", base_dir);
    match generate_region_config_ini(&config_path) {
        Ok(file) => file,
        Err(error) => panic!("Problem opening the file: {error:?}"),
    };

    let mut child = match Command::new("mono")
        .arg(format!("{}/bin/{}", base_dir, sim_executable))
        .current_dir(format!("{}/bin", base_dir))
        .stdout(Stdio::piped())
        .spawn()
    {
        Ok(child) => child,
        Err(e) => {
            error!(
                "could not start server at {}/bin/{}: {}",
                base_dir, sim_executable, e
            );
            return Err(format!("Could not start instance: {}", e));
        }
    };

    // Create a channel to receive output lines from the mono program
    let (tx, rx) = channel();

    // Spawn a thread to read stdout from the mono program
    thread::spawn(move || {
        if let Some(stdout) = child.stdout.take() {
            let reader = BufReader::new(stdout);
            for line in reader.lines() {
                if let Ok(line) = line {
                    // Send each line to the receiver
                    if tx.send(line).is_err() {
                        break; // channel closed
                    }
                }
            }
        }
    });

    // Return the receiver so the caller can receive output lines
    Ok(rx)
}

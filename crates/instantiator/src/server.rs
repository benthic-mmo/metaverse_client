use log::{info, warn};
use reqwest::Url;
use std::collections::HashMap;
use std::fs::File;
use std::io::{copy, Cursor, Read};
use std::path::{Path, PathBuf};

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

use metaverse_instantiator::config_generator::{
    create_default_config, create_default_standalone_config, create_full_config,
};
use metaverse_instantiator::server::{download_sim, read_sim_config, start_default_sim_instance};

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

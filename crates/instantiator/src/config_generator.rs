use crate::models::{conf_spec::*, region::RegionsConfig, standalone_spec::StandaloneConfig};
use std::path::PathBuf;

pub fn create_default_config() -> SimulatorConfig {
    SimulatorConfig {
        ..Default::default()
    }
}

pub fn create_default_standalone_config() -> StandaloneConfig {
    StandaloneConfig {
        ..Default::default()
    }
}

pub fn create_default_region_config() -> RegionsConfig {
    RegionsConfig {
        ..Default::default()
    }
}

//TODO: actually create the full config from the example ini. But it's a lot of work
pub fn create_full_config() -> SimulatorConfig {
    SimulatorConfig {
        config_const: ConfigConst {
            ..Default::default()
        },
        startup: Startup {
            console_prompt: Some("Region (\\R)".to_string()),
            console_history_file_enabled: Some(true),
            console_history_file: Some(PathBuf::from("OpenSimConsoleHistory.txt")),
            console_history_file_lines: Some(100),
            console_history_time_stamp: Some(false),
            save_crashes: Some(false),
            crash_dir: Some(PathBuf::from("crashes")),
            pid_file: Some(PathBuf::from("/tmp/OpenSim.exe.pid")),
            registry_location: Some(PathBuf::from(".")),
            config_directory: Some(PathBuf::from(".")),
            region_info_source: Some(RegionInfoSource::Web),
            region_load_regions_dir: Some(PathBuf::from("C:\\somewhere\\xmlfiles\\")),
            region_load_webserver_url: Some("http://example.com/regions.xml".to_string()),
            allow_regionless: Some(false),
            non_physical_prim_min: Some(0.001),
            non_physical_prim_max: Some(256_f32),
            physical_prim_min: Some(0.01),
            physical_prim_max: Some(64_f32),
            clamp_prim_size: Some(false),
            link_set_prims: Some(0 as f32),
            allow_script_crossing: Some(false),
            trust_binaries: Some(false),
            in_world_restart_shuts_down: Some(false),
            minimum_time_before_persistence_considered: Some(60),
            maximum_time_before_persistence_considered: Some(600),
            physical_prim: Some(true),
            meshing: Some(Mesher::Meshmerizer),
            physics: Some(PhysicsEngine::BulletSim),
            default_script_engine: Some(ScriptEngine::XEngine),
            http_proxy: Some("http://proxy.com:8080".to_string()),
            http_proxy_exceptions: Some(HttpProxyExceptionsList(vec![
                ".mydomain.com".to_string(),
                "localhost".to_string(),
            ])),
            email_module: Some("DefaultEmailModule".to_string()),
            spawn_point_routing: Some(SpawnPointRouting::Closest),
            tele_hub_allow_landmark: Some(false),
            no_verify_cert_chain: Some(false),
            no_verify_cert_host_name: Some(false),
        },
        ..Default::default()
    }
}

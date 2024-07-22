use crate::models::{conf_spec::*, region::RegionsConfig, standalone_spec::StandaloneConfig};

// generate config using defaults 
pub fn create_default_config() -> SimulatorConfig {
    SimulatorConfig {
        ..Default::default()
    }
}

// generate standalone config using defaults 
pub fn create_default_standalone_config() -> StandaloneConfig {
    StandaloneConfig {
        ..Default::default()
    }
}

// generate region config using defaults 
pub fn create_default_region_config() -> RegionsConfig {
    RegionsConfig {
        ..Default::default()
    }
}



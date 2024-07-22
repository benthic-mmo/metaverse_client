use std::fmt;
use uuid::Uuid;

pub struct Coordinate {
    x: i32,
    y: i32,
}
impl fmt::Display for Coordinate {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{},{}", &self.x, &self.y)
    }
}
impl Default for Coordinate {
    fn default() -> Self {
        Self { x: 1000, y: 1000 }
    }
}

// TODO: add the optional args here
pub struct Region {
    pub name: String,
    pub region_uuid: String,
    pub location: Coordinate,
    pub internal_address: String,
    pub internal_port: i32,
    pub allow_alternate_ports: bool,
    pub external_hostname: String,
    pub size_x: i32,
    pub size_y: i32,
}
impl fmt::Display for Region {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut result = String::new();

        result.push_str(&format!("[{}]", &self.name));
        result.push_str(&format!("RegionUUID = {}", &self.region_uuid));
        result.push_str(&format!("Location = {}", &self.location.to_string()));
        result.push_str(&format!("InternalAddress = {}", &self.internal_address));
        result.push_str(&format!("InternalPort = {}", &self.internal_port));
        result.push_str(&format!(
            "AllowAlternatePorts = {}",
            &self.allow_alternate_ports
        ));
        result.push_str(&format!("ExternalHostName = {}", &self.external_hostname));
        result.push_str(&format!("SizeX = {}", &self.size_x));
        result.push_str(&format!("SizeY = {}", &self.size_y));
        write!(f, "{}", result)
    }
}
impl Default for Region {
    fn default() -> Self {
        Self {
            name: "Default Region".to_string(),
            region_uuid: Uuid::new_v4().to_string(),
            location: Coordinate {
                ..Default::default()
            },
            internal_address: "0.0.0.0".to_string(),
            internal_port: 9000,
            allow_alternate_ports: false,
            external_hostname: "127.0.0.1".to_string(),
            size_x: 512,
            size_y: 512,
        }
    }
}

#[derive(Default)]
pub struct RegionsConfig(pub Vec<Region>);
impl fmt::Display for RegionsConfig {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut output = String::new();

        for region in &self.0 {
            output.push_str(&format!("{}, ", region));
        }
        write!(f, "{}", output)
    }
}

pub struct SimulatorLoginProtocol{
    pub first: String,
    pub last: String,
    pub passwd: String, 
    pub start: String, 
    pub channel: String, 
    pub version: String, 
    pub platform: String, 
    pub platform_string: String, 
    pub platform_version: String, 
    pub mac: String, 
    pub id0: String, 
    pub agree_to_tos: bool,
    pub read_critical: bool, 
    pub viewer_digest: String,
    pub address_size: String, 
    pub extended_errors: String, 
    pub last_exec_event: i64, 
    pub last_exec_duration: String, 
    pub skipoptional: bool,
    pub options: String,
}

pub fn hello() -> i64{
    return 2 + 2 
} 


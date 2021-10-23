pub struct SimulatorLoginProtocol{
    pub first:              String,
    pub last:               String,
    pub passwd:             String, 
    pub start:              String, 
    pub channel:            String, 
    pub version:            String, 
    pub platform:           String, 
    pub platform_string:    String, 
    pub platform_version:   String, 
    pub mac:                String, 
    pub id0:                String, 
    pub agree_to_tos:       bool,
    pub read_critical:      bool, 
    pub viewer_digest:      String,
    pub address_size:       String, 
    pub extended_errors:    String, 
    pub last_exec_event:    i64, 
    pub last_exec_duration: String, 
    pub skipoptional:       bool,
    pub options:            String,
}

pub struct SimulatorLoginOptions{
    pub adult_compliant:        String, 
    pub advanced_mode:          String, 
    pub avatar_picker_url:      String,
    pub buddy_list:             String,
    pub classified_categories:  String,
    pub currency:               String, 
    pub destination_guide_url:  String, 
    pub display_names:          String,
    pub event_categories:       String, 
    pub gestures:               String, 
    pub global_textures:        String, 
    pub inventory_root:         String, 
    pub inventory_sleketon:     String, 
    pub inventory_lib_root:     String, 
    pub inventory_lib_owner:    String, 
    pub inventory_skel_lib:     String, 
    pub login_flags:            String, 
    pub max_agent_groups:       String, 
    pub max_groups:             String, 
    pub map_server_url:         String,
    pub newuser_config:         String, 
    pub search:                 String, 
    pub tutorial_setting:       String, 
    pub ui_config:              String, 
    pub voice_config:           String 
}

impl Into<xmlrpc::Value> for SimulatorLoginProtocol{
    fn into(self) -> xmlrpc::Value{
        xmlrpc::Value::Struct(vec![
            ("first"                .to_string(), xmlrpc::Value::from(self.first.clone())), 
            ("last"                 .to_string(), xmlrpc::Value::from(self.last.clone())), 
            ("passwd"               .to_string(), xmlrpc::Value::from(self.passwd.clone())), 
            ("start"                .to_string(), xmlrpc::Value::from(self.start.clone())), 
            ("channel"              .to_string(), xmlrpc::Value::from(self.channel.clone())), 
            ("version"              .to_string(), xmlrpc::Value::from(self.version.clone())),
            ("platform"             .to_string(), xmlrpc::Value::from(self.version.clone())), 
            ("platform_string"      .to_string(), xmlrpc::Value::from(self.platform_string.clone())), 
            ("platform_version"     .to_string(), xmlrpc::Value::from(self.platform_version.clone())), 
            ("mac"                  .to_string(), xmlrpc::Value::from(self.mac.clone())), 
            ("id0"                  .to_string(), xmlrpc::Value::from(self.id0.clone())), 
            ("agree_to_tos"         .to_string(), xmlrpc::Value::from(self.agree_to_tos.clone())), 
            ("read_critical"        .to_string(), xmlrpc::Value::from(self.read_critical.clone())), 
            ("viewer_digest"        .to_string(), xmlrpc::Value::from(self.viewer_digest.clone())), 
            ("address_size"         .to_string(), xmlrpc::Value::from(self.address_size.clone())),
            ("last_exec_event"      .to_string(), xmlrpc::Value::from(self.last_exec_event.clone())),
            ("last_exec_duration"   .to_string(), xmlrpc::Value::from(self.last_exec_duration.clone())), 
            ("skipoptional"         .to_string(), xmlrpc::Value::from(self.skipoptional.clone())), 
            ("options"              .to_string(), xmlrpc::Value::from(self.options.clone()))
        ].into_iter().collect())
    } 
} 

pub fn hello() -> i64{
    return 2 + 2 
} 


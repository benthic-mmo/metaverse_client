///SimulatorLoginProtocol- the struct for creating a login call
///implemented from the protocol as defined by <http://opensimulator.org/wiki/SimulatorLoginProtocol>
#[derive(Clone, Default)]
pub struct SimulatorLoginProtocol {
    pub first: String,  //First name of the user
    pub last: String,   //Last name of the user
    pub passwd: String, //MD5 hash of fthe user's password with the string "$1$" prepended
    pub start: String,  //The region in which the user should start upon login. This can be one of
    //  "home" - start in the user's home location
    //  "last" - start in the location where the user last logged out
    //  a specific location - in the format
    //      uri:<region-name>&<x-coord>&<y-coord>&<z-coord>
    //      for example, the string "uri:test&128&128&0" says the user
    //      should log in  to the center of the region named test, and
    //      the avatar should be placed ono the ground if the
    //      z-coordinate is below the terrain.
    pub channel: Option<String>,  //Name of the viewer/client connecting
    pub version: Option<String>,  //Version of the viewer/client connecting
    pub platform: Option<String>, //Platform the viewer/client is connecting from. Can be one of
    //  lin - linux
    //  mac - mac
    //  win - windows
    pub platform_string: Option<String>, //The operating system description and version (e.g. "Linux 5.8, "Fedora 32")
    pub platform_version: Option<String>, //  operating system version
    pub mac: Option<String>, //The mac address of the network card used by the client/viewer
    pub id0: Option<String>, //A hardwarae hash basedo n the serial number of the user's first hard drive
    //  used to uniquely identify computers and track users
    pub agree_to_tos: Option<bool>, //Has user agreed to the terms of service. Boolean true/false
    pub read_critical: Option<bool>, //Has user read terms of service or other docs. Boolean true/false
    pub viewer_digest: Option<String>, //MD5 hash of theh viewer executable
    pub address_size: Option<String>, //TODO: figure out what this means
    pub extended_errors: Option<String>, //TODO: figure out what this means
    pub last_exec_event: Option<i64>, //TODO: figure out what this means
    pub last_exec_duration: Option<String>, //TODO: figure out what this means
    pub skipoptional: Option<bool>,  //TODO: figure out what this means :/
    pub options: Option<SimulatorLoginOptions>,
}

///SimulatorLoginOptions - the contents of the options field in the SimulatorLoginProtocol
///none of these are documented :(
///parameters seem to randomly swap between using _ and - to break up words.
///I've documented these fields in the struct and the Value impl
#[derive(Clone, Default)]
pub struct SimulatorLoginOptions {
    pub adult_compliant: Option<String>,
    pub advanced_mode: Option<String>,
    pub avatar_picker_url: Option<String>,
    pub buddy_list: Option<String>, //buddy-list in xml
    pub classified_categories: Option<String>,
    pub currency: Option<String>,
    pub destination_guide_url: Option<String>,
    pub display_names: Option<String>,
    pub event_categories: Option<String>,
    pub gestures: Option<String>,
    pub global_textures: Option<String>, //global-textures in xml
    pub inventory_root: Option<String>,  //inventory-root in xml
    pub inventory_skeleton: Option<String>, //inventory-skeleton in xml
    pub inventory_lib_root: Option<String>, //inventory-lib-root in xml
    pub inventory_lib_owner: Option<String>, //inventory-lib-owner in xml
    pub inventory_skel_lib: Option<String>, //inventory-skel-lib in xml
    pub login_flags: Option<String>,     //login-flags in xml
    pub max_agent_groups: Option<String>, //max-agent-groups in xml
    pub max_groups: Option<String>,
    pub map_server_url: Option<String>, //map-server-url in xml
    pub newuser_config: Option<String>, //newusuer-config in xml
    pub search: Option<String>,
    pub tutorial_setting: Option<String>, //tutorial-setting in xml
    pub ui_config: Option<String>,        //ui-config in xml
    pub voice_config: Option<String>,     //voice-config in xml
}

///Creates value type from a SimulatorLoginOption struct
fn value_from_option(other: Option<SimulatorLoginOptions>) -> xmlrpc::Value {
    match other {
        Some(x) => x.into(),
        None => xmlrpc::Value::Nil,
    }
}

///Converts the SimulatorLoginOptions into an xmlrpc::Value
///automatically removes empty options
impl Into<xmlrpc::Value> for SimulatorLoginOptions {
    fn into(self) -> xmlrpc::Value {
        let mut options_vec = vec![
            (
                "adult_compliant".to_string(),
                xmlrpc::Value::from(self.adult_compliant.clone()),
            ),
            (
                "advanced_mode".to_string(),
                xmlrpc::Value::from(self.advanced_mode.clone()),
            ),
            (
                "avatar_picker_url".to_string(),
                xmlrpc::Value::from(self.avatar_picker_url.clone()),
            ),
            (
                "buddy-list".to_string(),
                xmlrpc::Value::from(self.buddy_list.clone()),
            ),
            //buddy-list in xml, buddy_list in source
            (
                "classified_categories".to_string(),
                xmlrpc::Value::from(self.classified_categories.clone()),
            ),
            (
                "currency".to_string(),
                xmlrpc::Value::from(self.currency.clone()),
            ),
            (
                "destination_guide_url".to_string(),
                xmlrpc::Value::from(self.destination_guide_url.clone()),
            ),
            (
                "display_names".to_string(),
                xmlrpc::Value::from(self.display_names.clone()),
            ),
            (
                "event_categories".to_string(),
                xmlrpc::Value::from(self.event_categories.clone()),
            ),
            (
                "gestures".to_string(),
                xmlrpc::Value::from(self.gestures.clone()),
            ),
            (
                "global-textures".to_string(),
                xmlrpc::Value::from(self.global_textures.clone()),
            ),
            //global-textures in xml, global_textures in source
            (
                "inventory-root".to_string(),
                xmlrpc::Value::from(self.inventory_root.clone()),
            ),
            //inventory-root in xml, iniventory_root in source
            (
                "inventory-skeleton".to_string(),
                xmlrpc::Value::from(self.inventory_skeleton.clone()),
            ),
            //inventory-skeleton in xml, inventory_skeleton in source
            (
                "inventory-lib-root".to_string(),
                xmlrpc::Value::from(self.inventory_lib_root.clone()),
            ),
            //inventory-lib-root in xml, inventory_lib_root in source
            (
                "inventory-skel-lib".to_string(),
                xmlrpc::Value::from(self.inventory_skel_lib.clone()),
            ),
            //inventory-skel-lib in xml, inventory_skel_lib in source
            (
                "login-flags".to_string(),
                xmlrpc::Value::from(self.login_flags.clone()),
            ),
            //login-flags in xml, login_flags in source
            (
                "max-agent-groups".to_string(),
                xmlrpc::Value::from(self.max_agent_groups.clone()),
            ),
            //max-agent-groups in xml, max_agent_groups in source
            (
                "max_groups".to_string(),
                xmlrpc::Value::from(self.max_groups.clone()),
            ),
            (
                "map-server-url".to_string(),
                xmlrpc::Value::from(self.map_server_url.clone()),
            ),
            //map-server-url in xml, ma_server_url in source
            (
                "newuser-config".to_string(),
                xmlrpc::Value::from(self.newuser_config.clone()),
            ),
            //newuser-config in xml, newuser_config in source
            (
                "search".to_string(),
                xmlrpc::Value::from(self.search.clone()),
            ),
            (
                "tutorial_setting".to_string(),
                xmlrpc::Value::from(self.tutorial_setting.clone()),
            ),
            (
                "ui-config".to_string(),
                xmlrpc::Value::from(self.ui_config.clone()),
            ),
            //ui-config in xml, ui_config in source
            (
                "voice-config".to_string(),
                xmlrpc::Value::from(self.voice_config.clone()),
            ), //voice-config in xml, voice_config in source
        ];

        options_vec.retain(|i| i.1 != xmlrpc::Value::Nil);
        xmlrpc::Value::Struct(options_vec.into_iter().collect())
    }
}

///Converts a SimulatorLoginProtocol into an xmlrpc::Value
///automatically removes empty options
impl Into<xmlrpc::Value> for SimulatorLoginProtocol {
    fn into(self) -> xmlrpc::Value {
        let mut login_vec = vec![
            ("first".to_string(), xmlrpc::Value::from(self.first.clone())),
            ("last".to_string(), xmlrpc::Value::from(self.last.clone())),
            (
                "passwd".to_string(),
                xmlrpc::Value::from(self.passwd.clone()),
            ),
            ("start".to_string(), xmlrpc::Value::from(self.start.clone())),
            (
                "channel".to_string(),
                xmlrpc::Value::from(self.channel.clone()),
            ),
            (
                "version".to_string(),
                xmlrpc::Value::from(self.version.clone()),
            ),
            (
                "platform".to_string(),
                xmlrpc::Value::from(self.version.clone()),
            ),
            (
                "platform_string".to_string(),
                xmlrpc::Value::from(self.platform_string.clone()),
            ),
            (
                "platform_version".to_string(),
                xmlrpc::Value::from(self.platform_version.clone()),
            ),
            ("mac".to_string(), xmlrpc::Value::from(self.mac.clone())),
            ("id0".to_string(), xmlrpc::Value::from(self.id0.clone())),
            (
                "agree_to_tos".to_string(),
                xmlrpc::Value::from(self.agree_to_tos.clone()),
            ),
            (
                "read_critical".to_string(),
                xmlrpc::Value::from(self.read_critical.clone()),
            ),
            (
                "viewer_digest".to_string(),
                xmlrpc::Value::from(self.viewer_digest.clone()),
            ),
            (
                "address_size".to_string(),
                xmlrpc::Value::from(self.address_size.clone()),
            ),
            (
                "last_exec_event".to_string(),
                xmlrpc::Value::from(self.last_exec_event.clone()),
            ),
            (
                "last_exec_duration".to_string(),
                xmlrpc::Value::from(self.last_exec_duration.clone()),
            ),
            (
                "skipoptional".to_string(),
                xmlrpc::Value::from(self.skipoptional.clone()),
            ),
            ("options".to_string(), value_from_option(self.options)),
        ];
        login_vec.retain(|i| i.1 != xmlrpc::Value::Nil);
        xmlrpc::Value::Struct(login_vec.into_iter().collect())
    }
}

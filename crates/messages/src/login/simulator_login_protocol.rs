use xmlrpc_benthic::{self as xmlrpc};

///SimulatorLoginProtocol- the struct for creating a login call
///implemented from the protocol as defined by <http://opensimulator.org/wiki/SimulatorLoginProtocol>
///
///This parsing may be causing long load times for login.
///this should be refactored to only parse small portions of this enormous struct.

#[derive(Clone, Default, Debug)]
pub struct SimulatorLoginProtocol {
    /// First name of the user
    pub first: String,
    /// Last name of the user
    pub last: String,
    /// MD5 hash of the user's password with the string "$1$" prepended
    pub passwd: String,
    /// The region in which the user should start upon login. This can be one of
    /// "home" - start in the user's home location
    /// "last" - start in the location where the user last logged out
    /// a specific location - in the format
    /// uri:`<region-name>&<x-coord>&<y-coord>&<z-coord>`
    /// for example, the string "uri:test&128&128&0" says the user
    /// should log in to the center of the region named test, and
    /// the avatar should be placed on the ground if the
    /// z-coordinate is below the terrain.
    pub start: String,
    /// Name of the viewer/client connecting
    pub channel: String,
    /// Version of the viewer/client connecting
    pub version: String,
    /// Platform the viewer/client is connecting from. Can be one of
    /// lin - linux
    /// mac - mac
    /// win - windows
    pub platform: String,
    /// The operating system description and version (e.g. "Linux 5.8", "Fedora 32")
    pub platform_string: String,
    /// Operating system version
    pub platform_version: String,
    /// The MAC address of the network card used by the client/viewer
    pub mac: String,
    /// A hardware hash based on the serial number of the user's first hard drive
    /// Used to uniquely identify computers and track users
    pub id0: String,
    /// Has user agreed to the terms of service. Boolean true/false
    pub agree_to_tos: bool,
    /// Has user read terms of service or other docs. Boolean true/false
    pub read_critical: bool,
    /// MD5 hash of the viewer executable
    pub viewer_digest: Option<String>,
    /// TODO: Figure out what this means
    pub address_size: i32,
    /// TODO: Figure out what this means
    pub extended_errors: bool,
    /// TODO: Figure out what this means
    pub last_exec_event: Option<i32>,
    /// TODO: Figure out what this means
    pub last_exec_duration: i32,
    /// TODO: Figure out what this means :/
    pub skipoptional: Option<bool>,
    /// TODO: Undocumented also
    pub host_id: String,
    /// TODO: Undocumented
    pub mfa_hash: String,
    /// undocumented
    pub token: String,
    /// undocumented
    pub options: SimulatorLoginOptions,
}
/// SimulatorLoginOptions - the contents of the options field in the SimulatorLoginProtocol
/// none of these are documented :(
/// parameters seem to randomly swap between using _ and - to break up words.
/// I've documented these fields in the struct and the Value impl

#[derive(Clone, Default, Debug)]
pub struct SimulatorLoginOptions {
    /// unused
    pub adult_compliant: Option<bool>,
    /// unused
    pub advanced_mode: Option<bool>,
    /// unused
    pub avatar_picker_url: Option<bool>,
    /// Buddy-list in XML
    pub buddy_list: Option<bool>,
    /// undocumented
    pub classified_categories: Option<bool>,
    /// The currency used by the sim
    pub currency: Option<bool>,
    /// unused
    pub destination_guide_url: Option<bool>,
    /// unused
    pub display_names: Option<bool>,
    /// unused
    pub event_categories: Option<bool>,
    /// unused
    pub gestures: Option<bool>,
    /// Global-textures in XML
    pub global_textures: Option<bool>,
    /// Inventory-root in XML
    pub inventory_root: Option<bool>,
    /// Inventory-skeleton in XML
    pub inventory_skeleton: Option<bool>,
    /// Inventory-lib-root in XML
    pub inventory_lib_root: Option<bool>,
    /// Inventory-lib-owner in XML
    pub inventory_lib_owner: Option<bool>,
    /// Inventory-skel-lib in XML
    pub inventory_skel_lib: Option<bool>,
    /// Login-flags in XML
    pub login_flags: Option<bool>,
    /// Max-agent-groups in XML
    pub max_agent_groups: Option<bool>,
    /// unused
    pub max_groups: Option<bool>,
    /// Map-server-url in XML
    pub map_server_url: Option<bool>,
    /// Newuser-config in XML
    pub newuser_config: Option<bool>,
    /// unused
    pub search: Option<bool>,
    /// Tutorial-setting in XML
    pub tutorial_setting: Option<bool>,
    /// UI-config in XML
    pub ui_config: Option<bool>,
    /// Voice-config in XML
    pub voice_config: Option<bool>,
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
impl From<SimulatorLoginOptions> for xmlrpc::Value {
    fn from(val: SimulatorLoginOptions) -> Self {
        let mut options_vec: Vec<String> = Vec::new();

        if let Some(value) = val.adult_compliant {
            if value {
                options_vec.push("adult_compliant".to_string());
            }
        }

        if let Some(value) = val.advanced_mode {
            if value {
                options_vec.push("advanced_mode".to_string());
            }
        }

        if let Some(value) = val.avatar_picker_url {
            if value {
                options_vec.push("avatar_picker_url".to_string());
            }
        }

        if let Some(value) = val.buddy_list {
            if value {
                options_vec.push("buddy-list".to_string());
            }
        }

        if let Some(value) = val.classified_categories {
            if value {
                options_vec.push("classified_categories".to_string());
            }
        }

        if let Some(value) = val.currency {
            if value {
                options_vec.push("currency".to_string());
            }
        }

        if let Some(value) = val.destination_guide_url {
            if value {
                options_vec.push("destination_guide_url".to_string());
            }
        }

        if let Some(value) = val.display_names {
            if value {
                options_vec.push("display_names".to_string());
            }
        }

        if let Some(value) = val.event_categories {
            if value {
                options_vec.push("event_categories".to_string());
            }
        }

        if let Some(value) = val.gestures {
            if value {
                options_vec.push("gestures".to_string());
            }
        }

        if let Some(value) = val.global_textures {
            if value {
                options_vec.push("global-textures".to_string());
            }
        }

        if let Some(value) = val.inventory_root {
            if value {
                options_vec.push("inventory-root".to_string());
            }
        }

        if let Some(value) = val.inventory_skeleton {
            if value {
                options_vec.push("inventory-skeleton".to_string());
            }
        }

        if let Some(value) = val.inventory_lib_root {
            if value {
                options_vec.push("inventory-lib-root".to_string());
            }
        }

        if let Some(value) = val.inventory_lib_owner {
            if value {
                options_vec.push("inventory-lib-owner".to_string());
            }
        }

        if let Some(value) = val.inventory_skel_lib {
            if value {
                options_vec.push("inventory-skel-lib".to_string());
            }
        }

        if let Some(value) = val.login_flags {
            if value {
                options_vec.push("login-flags".to_string());
            }
        }

        if let Some(value) = val.max_agent_groups {
            if value {
                options_vec.push("max-agent-groups".to_string());
            }
        }

        if let Some(value) = val.max_groups {
            if value {
                options_vec.push("max_groups".to_string());
            }
        }

        if let Some(value) = val.map_server_url {
            if value {
                options_vec.push("map-server-url".to_string());
            }
        }

        if let Some(value) = val.newuser_config {
            if value {
                options_vec.push("newuser-config".to_string());
            }
        }

        if let Some(value) = val.search {
            if value {
                options_vec.push("search".to_string());
            }
        }

        if let Some(value) = val.tutorial_setting {
            if value {
                options_vec.push("tutorial_setting".to_string());
            }
        }

        if let Some(value) = val.ui_config {
            if value {
                options_vec.push("ui-config".to_string());
            }
        }

        if let Some(value) = val.voice_config {
            if value {
                options_vec.push("voice-config".to_string());
            }
        }
        let xmlrpc_array: xmlrpc::Value =
            xmlrpc::Value::Array(options_vec.into_iter().map(xmlrpc::Value::String).collect());
        xmlrpc_array
    }
}

/// converts an xmlrpc boolean to a real int
fn bool_to_int(value: bool) -> xmlrpc::Value {
    if value {
        xmlrpc::Value::Int(1)
    } else {
        xmlrpc::Value::Int(0)
    }
}

///Converts a SimulatorLoginProtocol into an xmlrpc::Value
///automatically removes empty options
impl From<SimulatorLoginProtocol> for xmlrpc::Value {
    fn from(val: SimulatorLoginProtocol) -> Self {
        let mut login_vec = vec![
            ("first".to_string(), xmlrpc::Value::from(val.first)),
            ("last".to_string(), xmlrpc::Value::from(val.last)),
            ("passwd".to_string(), xmlrpc::Value::from(val.passwd)),
            ("start".to_string(), xmlrpc::Value::from(val.start)),
            ("channel".to_string(), xmlrpc::Value::from(val.channel)),
            ("version".to_string(), xmlrpc::Value::from(val.version)),
            ("platform".to_string(), xmlrpc::Value::from(val.platform)),
            (
                "platform_string".to_string(),
                xmlrpc::Value::from(val.platform_string),
            ),
            (
                "platform_version".to_string(),
                xmlrpc::Value::from(val.platform_version),
            ),
            ("mac".to_string(), xmlrpc::Value::from(val.mac)),
            ("id0".to_string(), xmlrpc::Value::from(val.id0)),
            ("agree_to_tos".to_string(), bool_to_int(val.agree_to_tos)),
            ("read_critical".to_string(), bool_to_int(val.read_critical)),
            (
                "viewer_digest".to_string(),
                xmlrpc::Value::from(val.viewer_digest),
            ),
            (
                "address_size".to_string(),
                xmlrpc::Value::from(val.address_size),
            ),
            (
                "extended_errors".to_string(),
                bool_to_int(val.extended_errors),
            ),
            (
                "last_exec_event".to_string(),
                xmlrpc::Value::from(val.last_exec_event),
            ),
            (
                "last_exec_duration".to_string(),
                xmlrpc::Value::from(val.last_exec_duration),
            ),
            (
                "skipoptional".to_string(),
                xmlrpc::Value::from(val.skipoptional),
            ),
            ("host_id".to_string(), xmlrpc::Value::from(val.host_id)),
            ("mfa_hash".to_string(), xmlrpc::Value::from(val.mfa_hash)),
            ("token".to_string(), xmlrpc::Value::from(val.token)),
            ("options".to_string(), value_from_option(Some(val.options))),
        ];

        login_vec.retain(|i| i.1 != xmlrpc::Value::Nil);
        xmlrpc::Value::Struct(login_vec.into_iter().collect())
    }
}

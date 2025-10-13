use crate::ui::login_event::Login;
use mac_address::get_mac_address;
use md5::{Digest, Md5};
use serde::{Deserialize, Serialize};
use serde_llsd::{ser::xml_rpc, LLSDValue};
use std::collections::HashMap;
use std::env;
use std::error::Error;
use std::fs::File;
use std::io::Read;

///SimulatorLoginProtocol- the struct for creating a login call
///implemented from the protocol as defined by <http://opensimulator.org/wiki/SimulatorLoginProtocol>
///
///This parsing may be causing long load times for login.
///this should be refactored to only parse small portions of this enormous struct.

#[derive(Clone, Default, Debug, Serialize, Deserialize)]
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

impl SimulatorLoginProtocol {
    /// Convert the login struct into an LLSDValue::Map suitable for XML-RPC
    pub fn to_llsd_map(&self) -> LLSDValue {
        let mut map = HashMap::new();

        map.insert("first".to_string(), LLSDValue::String(self.first.clone()));
        map.insert("last".to_string(), LLSDValue::String(self.last.clone()));
        map.insert("passwd".to_string(), LLSDValue::String(self.passwd.clone()));
        map.insert("start".to_string(), LLSDValue::String(self.start.clone()));
        map.insert(
            "channel".to_string(),
            LLSDValue::String(self.channel.clone()),
        );
        map.insert(
            "version".to_string(),
            LLSDValue::String(self.version.clone()),
        );
        map.insert(
            "platform".to_string(),
            LLSDValue::String(self.platform.clone()),
        );
        map.insert(
            "platform_string".to_string(),
            LLSDValue::String(self.platform_string.clone()),
        );
        map.insert(
            "platform_version".to_string(),
            LLSDValue::String(self.platform_version.clone()),
        );
        map.insert("mac".to_string(), LLSDValue::String(self.mac.clone()));
        map.insert("id0".to_string(), LLSDValue::String(self.id0.clone()));
        map.insert(
            "agree_to_tos".to_string(),
            LLSDValue::Boolean(self.agree_to_tos),
        );
        map.insert(
            "read_critical".to_string(),
            LLSDValue::Boolean(self.read_critical),
        );

        if let Some(digest) = &self.viewer_digest {
            map.insert(
                "viewer_digest".to_string(),
                LLSDValue::String(digest.clone()),
            );
        }

        map.insert(
            "address_size".to_string(),
            LLSDValue::Integer(self.address_size),
        );
        map.insert(
            "extended_errors".to_string(),
            LLSDValue::Boolean(self.extended_errors),
        );

        if let Some(last) = self.last_exec_event {
            map.insert("last_exec_event".to_string(), LLSDValue::Integer(last));
        }

        map.insert(
            "last_exec_duration".to_string(),
            LLSDValue::Integer(self.last_exec_duration),
        );

        if let Some(skip) = self.skipoptional {
            map.insert("skipoptional".to_string(), LLSDValue::Boolean(skip));
        }

        map.insert(
            "host_id".to_string(),
            LLSDValue::String(self.host_id.clone()),
        );
        map.insert(
            "mfa_hash".to_string(),
            LLSDValue::String(self.mfa_hash.clone()),
        );
        map.insert("token".to_string(), LLSDValue::String(self.token.clone()));

        // Convert options (assuming SimulatorLoginOptions implements a `to_llsd` method)
        map.insert("options".to_string(), self.options.to_llsd());

        LLSDValue::Map(map)
    }
}
/// SimulatorLoginOptions - the contents of the options field in the SimulatorLoginProtocol
/// none of these are documented :(
/// parameters seem to randomly swap between using _ and - to break up words.
/// I've documented these fields in the struct and the Value impl

#[derive(Clone, Default, Debug, Serialize, Deserialize)]
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
impl SimulatorLoginOptions {
    /// Convert the login options to LLSDValue::Array of strings
    pub fn to_llsd(&self) -> LLSDValue {
        let mut vec = Vec::new();

        if self.inventory_root.unwrap_or(false) {
            vec.push(LLSDValue::String("inventory-root".to_string()));
        }
        if self.inventory_skeleton.unwrap_or(false) {
            vec.push(LLSDValue::String("inventory-skeleton".to_string()));
        }
        if self.inventory_lib_root.unwrap_or(false) {
            vec.push(LLSDValue::String("inventory-lib-root".to_string()));
        }
        if self.inventory_lib_owner.unwrap_or(false) {
            vec.push(LLSDValue::String("inventory-lib-owner".to_string()));
        }
        if self.inventory_skel_lib.unwrap_or(false) {
            vec.push(LLSDValue::String("inventory-skel-lib".to_string()));
        }
        if self.gestures.unwrap_or(false) {
            vec.push(LLSDValue::String("gestures".to_string()));
        }
        if self.event_categories.unwrap_or(false) {
            vec.push(LLSDValue::String("event_categories".to_string()));
        }
        if self.classified_categories.unwrap_or(false) {
            vec.push(LLSDValue::String("classified_categories".to_string()));
        }
        if self.adult_compliant.unwrap_or(false) {
            vec.push(LLSDValue::String("adult_compliant".to_string()));
        }
        if self.buddy_list.unwrap_or(false) {
            vec.push(LLSDValue::String("buddy-list".to_string()));
        }
        if self.global_textures.unwrap_or(false) {
            vec.push(LLSDValue::String("global-textures".to_string()));
        }
        if self.login_flags.unwrap_or(false) {
            vec.push(LLSDValue::String("login-flags".to_string()));
        }
        if self.max_agent_groups.unwrap_or(false) {
            vec.push(LLSDValue::String("max-agent-groups".to_string()));
        }

        LLSDValue::Array(vec)
    }
}

impl SimulatorLoginProtocol {
    pub fn to_xml(self) -> String {
        let llsd = self.to_llsd_map();
        xml_rpc::to_string(&llsd, false, "login_to_simulator").unwrap()
    }
    pub fn new(login: Login) -> Self {
        SimulatorLoginProtocol {
            first: login.first,
            last: login.last,
            passwd: hash_passwd(login.passwd),
            start: login.start,
            channel: login.channel,
            version: env!("CARGO_PKG_VERSION").to_string(),
            platform: match env::consts::FAMILY {
                "mac" => "mac".to_string(),
                "win" => "win".to_string(),
                "unix" => "lin".to_string(),
                _ => "lin".to_string(),
            },
            platform_string: sys_info::os_release().unwrap_or_default(),
            platform_version: sys_info::os_release().unwrap_or_default(),
            mac: match get_mac_address() {
                Ok(Some(mac)) => format!("{}", mac),
                _ => format!("{}", 00000000000000000000000000000000),
            },
            id0: "unused".to_string(), // Provide a default value for id0. This is unused by default
            agree_to_tos: login.agree_to_tos,
            read_critical: login.read_critical,
            viewer_digest: match hash_viewer_digest() {
                Ok(viewer_digest) => Some(viewer_digest),
                Err(_) => Some("unused".to_string()),
            },
            address_size: 64,         // Set a default value if needed
            extended_errors: true,    // Set a default value if needed
            last_exec_event: None,    // Default to None
            last_exec_duration: 0,    // Set a default value if needed
            skipoptional: None,       // Default to None
            host_id: "".to_string(),  // Set a default value if needed
            mfa_hash: "".to_string(), // Set a default value if needed
            token: "".to_string(),    // Set a default value if needed
            options: SimulatorLoginOptions::default(), // Use default options
        }
    }
}

/// md5 hashes the password
fn hash_passwd(passwd_raw: String) -> String {
    let mut hasher = md5::Md5::new();
    hasher.update(passwd_raw);
    format!("$1${:x}", hasher.finalize())
}

/// Creates the viewer digest, a fingerprint of the viewer executable
/// this isn't used by opensimulator, but it's fun to have
fn hash_viewer_digest() -> Result<String, Box<dyn Error>> {
    let path = env::args().next().ok_or("No argument found")?;
    let mut f = File::open(path)?;
    let mut byt = Vec::new();
    f.read_to_end(&mut byt)?;

    let mut hasher = Md5::new();
    hasher.update(&byt);
    let hash = hasher.finalize();

    Ok(format!("{:x}", hash))
}

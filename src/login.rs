use crate::models::simulator_login_protocol::{SimulatorLoginOptions, SimulatorLoginProtocol};
use std::env;

extern crate mac_address;
extern crate sys_info;

///Logs in using a SimulatorLoginProtocol object and the url string.
///returns an xmlrpc::Value containing the server's response
///
///# Examples
///
///this test always fails because there is no xmlrpc server running
///see tests/login.rs, test_lib_osgrid_connect, test_lib_osgrid_minimal for more information
///```
///use metaverse_login::models::simulator_login_protocol::{SimulatorLoginProtocol};
///use metaverse_login::login::{login};
///use std::panic;
///
///let url = "http://127.0.0.1:80";
///let login_data: SimulatorLoginProtocol = SimulatorLoginProtocol {
///     first: "first".to_string(),
///     last: "last".to_string(),
///     passwd: "password".to_string(),
///     start: "home".to_string(),
///     ..SimulatorLoginProtocol::default()
///};
///
///panic::catch_unwind(|| login(login_data, url.to_string()));
///```
pub fn login(login_data: SimulatorLoginProtocol, url: String) -> xmlrpc::Value {
    let req = xmlrpc::Request::new("login_to_simulator").arg(login_data);
    req.call_url(&url).unwrap()
}

///Logs in using a generated SimulatorLoginProtocol object
///returns an xmlrpc::Value containing the server's response
///
///# Examples
///
///this test always fails because there is no xmlrpc server running
///see tests/login.rs test_lib_build_struct_with_defaults for more information
///```
/// use metaverse_login::login::{login_with_defaults};
/// use std::panic;
///
/// let url = "http://127.0.0.1:80";
/// panic::catch_unwind(||login_with_defaults(
///                         "first".to_string(),
///                         "last".to_string(),
///                         "password".to_string(),
///                         "home".to_string(),
///                         true,
///                         true,
///                         url.to_string()));
///```
pub fn login_with_defaults(
    first: String,
    last: String,
    passwd: String,
    start: String,
    agree_to_tos: bool,
    read_critical: bool,
    url: String,
) -> xmlrpc::Value {
    let login_data =
        build_struct_with_defaults(first, last, passwd, start, agree_to_tos, read_critical);
    let req = xmlrpc::Request::new("login_to_simulator").arg(login_data);
    req.call_url(&url).unwrap()
}

///Generates a SimulatorLoginProtocol based on runtime values
///returns a SimulatorLoginProtocol
///```
///use metaverse_login::login::{build_struct_with_defaults};
///
///let login_struct = build_struct_with_defaults(
///         "first".to_string(),
///         "last".to_string(),
///         "passwd".to_string(),
///         "home".to_string(),
///         true,
///         true);
///assert_eq!(login_struct.first, "first");
///```
///
pub fn build_struct_with_defaults(
    first: String,
    last: String,
    passwd: String,
    start: String,
    agree_to_tos: bool,
    read_critical: bool,
) -> SimulatorLoginProtocol {
    SimulatorLoginProtocol {
        first,
        last,
        passwd,
        start,
        channel: Some(env!("CARGO_CRATE_NAME").to_string()),
        version: Some(env!("CARGO_PKG_VERSION").to_string()),
        platform: Some(match env::consts::FAMILY {
            "mac" => "mac".to_string(),
            "win" => "win".to_string(),
            "unix" => "lin".to_string(),
            _ => "lin".to_string(),
        }),
        platform_string: Some(sys_info::os_release().unwrap()),
        platform_version: Some(sys_info::os_release().unwrap()),
        mac: Some(mac_address::get_mac_address().unwrap().unwrap().to_string()),
        agree_to_tos: Some(agree_to_tos),
        read_critical: Some(read_critical),
        ..SimulatorLoginProtocol::default()
    }
}

///Generates a SimulatorLoginProtocol from explicitly defined values
///returns a SimulatorLoginProtocol
pub fn build_login(
    first: String,
    last: String,
    passwd: String,
    start: String,
    channel: Option<String>,
    version: Option<String>,
    platform: Option<String>,
    platform_string: Option<String>,
    platform_version: Option<String>,
    mac: Option<String>,
    id0: Option<String>,
    agree_to_tos: Option<bool>,
    read_critical: Option<bool>,
    viewer_digest: Option<String>,
    address_size: Option<String>,
    extended_errors: Option<String>,
    last_exec_event: Option<i64>,
    last_exec_duration: Option<String>,
    skipoptional: Option<bool>,
    adult_compliant: Option<String>,
    advanced_mode: Option<String>,
    avatar_picker_url: Option<String>,
    buddy_list: Option<String>,
    classified_categories: Option<String>,
    currency: Option<String>,
    destination_guide_url: Option<String>,
    display_names: Option<String>,
    event_categories: Option<String>,
    gestures: Option<String>,
    global_textures: Option<String>,
    inventory_root: Option<String>,
    inventory_skeleton: Option<String>,
    inventory_lib_root: Option<String>,
    inventory_lib_owner: Option<String>,
    inventory_skel_lib: Option<String>,
    login_flags: Option<String>,
    max_agent_groups: Option<String>,
    max_groups: Option<String>,
    map_server_url: Option<String>,
    newuser_config: Option<String>,
    search: Option<String>,
    tutorial_setting: Option<String>,
    ui_config: Option<String>,
    voice_config: Option<String>,
) -> SimulatorLoginProtocol {
    SimulatorLoginProtocol {
        first,
        last,
        passwd,
        start,
        channel,
        version,
        platform,
        platform_string,
        platform_version,
        mac,
        id0,
        agree_to_tos,
        read_critical,
        viewer_digest,
        address_size,
        extended_errors,
        last_exec_event,
        last_exec_duration,
        skipoptional,
        options: Some(SimulatorLoginOptions {
            adult_compliant,
            advanced_mode,
            avatar_picker_url,
            buddy_list,
            classified_categories,
            currency,
            destination_guide_url,
            display_names,
            event_categories,
            gestures,
            global_textures,
            inventory_root,
            inventory_skeleton,
            inventory_lib_root,
            inventory_lib_owner,
            inventory_skel_lib,
            login_flags,
            max_agent_groups,
            max_groups,
            map_server_url,
            newuser_config,
            search,
            tutorial_setting,
            ui_config,
            voice_config,
        }),
        ..SimulatorLoginProtocol::default()
    }
}

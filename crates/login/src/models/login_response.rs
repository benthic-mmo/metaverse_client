use std::fmt;
use uuid::Uuid;
use std::error::Error;

pub enum LoginResult{
    Success(LoginResponse),
    Failure(LoginFailure),
}

pub struct LoginFailure {
    pub login: String,
    pub message: String,
    pub reason: String,
}


#[derive(Clone, Default, Debug)]
pub struct LoginResponse {
    pub home: Option<HomeValues>, // the home location of the user
    pub look_at: Option<(String, String, String)>, // the direction the avatar should be facing
    // This is a unit vector so
    // (0, 1, 0) is facing straight north,
    // (1, 0, 0) is east,
    // (0,-1, 0) is south and
    // (-1, 0, 0) is west.
    pub agent_access: Option<AgentAccess>, // The current maturity access level of the user
    pub agent_access_max: Option<AgentAccess>, // THe maximum level of region the user can access
    pub seed_capability: Option<String>, // The URL that the viewer should use to request further capabilities
    pub first_name: String,      // The first name of the user
    pub last_name: String,       // The last name of the user
    pub agent_id: Option<Uuid>,          // The id of the user
    pub sim_ip: Option<String>,          // The ip used to communicate with the recieving simulator
    pub sim_port: Option<u16>, // The UDP port used to communicate with the receiving simulator
    pub http_port: Option<u16>, // function unknown. Always set to 0 by OpenSimulator
    pub start_location: Option<String>, // The location where the user starts on login. "last", "home" or region location
    pub region_x: Option<i64>,          //The x grid coordinate of the start region in meters.
    // so a region at map co-ordinate 1000 will have a grid co-ordinate of 256000.
    pub region_y: Option<i64>, // the y grid coordinate of the start region in meters
    pub region_size_x: Option<i64>, // the size of the start region in meters.
    // usually will be 236 but with a varregion this can be a multiple
    // of 256
    pub region_size_y: Option<i64>,
    pub circuit_code: u32, // Circuit code to use for all UDP connections
    pub session_id: Option<Uuid>,  //UUID of this session
    pub secure_session_id: Option<Uuid>, //the secure UUID of this session
    pub inventory_root: Option<Vec<InventoryRootValues>>, // the ID of the user's root folder
    pub inventory_skeleton: Option<Vec<InventorySkeletonValues>>, // details about the child folders of the root folder.
    pub inventory_lib_root: Option<Vec<InventoryRootValues>>, // the ID of the library root folder
    pub inventory_skeleton_lib: Option<Vec<InventorySkeletonValues>>, //details about the child folders of the library root folder
    pub inventory_lib_owner: Option<Vec<AgentID>>, // the ID of the user that owns the library
    pub map_server_url: Option<String>,            //URL from which to request map tiles
    pub buddy_list: Option<Vec<BuddyListValues>>, // the user's friend list. Contains an entry for each friend
    pub gestures: Option<Vec<GesturesValues>>,    // the gestures the user currently has active.
    pub initial_outfit: Option<Vec<InitialOutfit>>, // use unknown, probably obsolete
    pub global_textures: Option<Vec<GlobalTextures>>, // use unknown, probably obsolete
    pub login: Option<bool>,                      // if logged in (should be true)
    pub login_flags: Option<Vec<LoginFlags>>,
    pub message: Option<String>,
    pub ui_config: Option<Vec<UiConfig>>,
    pub event_categories: Option<String>, // unknown
    pub classified_categories: Option<Vec<ClassifiedCategoriesValues>>,
    pub real_id: Option<String>,               //TODO: NOT DOCUMENTED!!
    pub search: Option<String>,                //TODO: NOT DOCUMENTED!!
    pub destination_guide_url: Option<String>, //TODO: NOT DOCUMENTED!!
    pub event_notifications: Option<String>,   //TODO: NOT DOCUMENTED!!
    pub max_agent_groups: Option<i64>,         //TODO: NOT DOCUMENTED!!
    pub seconds_since_epoch: Option<i64>,      //TODO: NOT DOCUMENTED!!
}
impl fmt::Display for LoginResponse {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut output = String::new();

        if let Some(home) = &self.home {
            output.push_str(&format!("home: {:?}\n", home));
        }
        if let Some(look_at) = &self.look_at {
            output.push_str(&format!("look_at: {:?}\n", look_at));
        }
        if let Some(agent_access) = &self.agent_access {
            output.push_str(&format!("agent_access: {:?}\n", agent_access));
        }
        if let Some(agent_access_max) = &self.agent_access_max {
            output.push_str(&format!("agent_access_max: {:?}\n", agent_access_max));
        }
        if let Some(seed_capability) = &self.seed_capability {
            output.push_str(&format!("seed_capability: {}\n", seed_capability));
        }
            output.push_str(&format!("first_name: {}\n", self.first_name));
            output.push_str(&format!("last_name: {}\n", self.last_name));
       
        if let Some(agent_id) = &self.agent_id {
            output.push_str(&format!("agent_id: {}\n", agent_id));
        }
        if let Some(sim_ip) = &self.sim_ip {
            output.push_str(&format!("sim_ip: {}\n", sim_ip));
        }
        if let Some(sim_port) = &self.sim_port {
            output.push_str(&format!("sim_port: {}\n", sim_port));
        }
        if let Some(http_port) = &self.http_port {
            output.push_str(&format!("http_port: {}\n", http_port));
        }
        if let Some(start_location) = &self.start_location {
            output.push_str(&format!("start_location: {}\n", start_location));
        }
        if let Some(region_x) = &self.region_x {
            output.push_str(&format!("region_x: {}\n", region_x));
        }
        if let Some(region_y) = &self.region_y {
            output.push_str(&format!("region_y: {}\n", region_y));
        }
        if let Some(region_size_x) = &self.region_size_x {
            output.push_str(&format!("region_size_x: {}\n", region_size_x));
        }
        if let Some(region_size_y) = &self.region_size_y {
            output.push_str(&format!("region_size_y: {}\n", region_size_y));
        }
            output.push_str(&format!("circuit_code: {}\n", self.circuit_code));

        if let Some(session_id) = &self.session_id {
            output.push_str(&format!("session_id: {}\n", session_id));
        }
        if let Some(secure_session_id) = &self.secure_session_id {
            output.push_str(&format!("secure_session_id: {}\n", secure_session_id));
        }
        if let Some(inventory_root) = &self.inventory_root {
            output.push_str(&format!("inventory_root: {:?}\n", inventory_root));
        }
        if let Some(inventory_skeleton) = &self.inventory_skeleton {
            output.push_str(&format!("inventory_skeleton: {:?}\n", inventory_skeleton));
        }
        if let Some(inventory_lib_root) = &self.inventory_lib_root {
            output.push_str(&format!("inventory_lib_root: {:?}\n", inventory_lib_root));
        }
        if let Some(inventory_skeleton_lib) = &self.inventory_skeleton_lib {
            output.push_str(&format!(
                "inventory_skeleton_lib: {:?}\n",
                inventory_skeleton_lib
            ));
        }
        if let Some(inventory_lib_owner) = &self.inventory_lib_owner {
            output.push_str(&format!("inventory_lib_owner: {:?}\n", inventory_lib_owner));
        }
        if let Some(map_server_url) = &self.map_server_url {
            output.push_str(&format!("map_server_url: {}\n", map_server_url));
        }
        if let Some(buddy_list) = &self.buddy_list {
            output.push_str(&format!("buddy_list: {:?}\n", buddy_list));
        }
        if let Some(gestures) = &self.gestures {
            output.push_str(&format!("gestures: {:?}\n", gestures));
        }
        if let Some(initial_outfit) = &self.initial_outfit {
            output.push_str(&format!("initial_outfit: {:?}\n", initial_outfit));
        }
        if let Some(global_textures) = &self.global_textures {
            output.push_str(&format!("global_textures: {:?}\n", global_textures));
        }
        if let Some(login) = &self.login {
            output.push_str(&format!("login: {}\n", login));
        }
        if let Some(login_flags) = &self.login_flags {
            output.push_str(&format!("login_flags: {:?}\n", login_flags));
        }
        if let Some(message) = &self.message {
            output.push_str(&format!("message: {}\n", message));
        }
        if let Some(ui_config) = &self.ui_config {
            output.push_str(&format!("ui_config: {:?}\n", ui_config));
        }
        if let Some(event_categories) = &self.event_categories {
            output.push_str(&format!("event_categories: {}\n", event_categories));
        }
        if let Some(classified_categories) = &self.classified_categories {
            output.push_str(&format!(
                "classified_categories: {:?}\n",
                classified_categories
            ));
        }
        if let Some(real_id) = &self.real_id {
            output.push_str(&format!("real_id: {}\n", real_id));
        }
        if let Some(search) = &self.search {
            output.push_str(&format!("search: {}\n", search));
        }
        if let Some(destination_guide_url) = &self.destination_guide_url {
            output.push_str(&format!(
                "destination_guide_url: {}\n",
                destination_guide_url
            ));
        }
        if let Some(event_notifications) = &self.event_notifications {
            output.push_str(&format!("event_notifications: {}\n", event_notifications));
        }
        if let Some(max_agent_groups) = &self.max_agent_groups {
            output.push_str(&format!("max_agent_groups: {}\n", max_agent_groups));
        }
        if let Some(seconds_since_epoch) = &self.seconds_since_epoch {
            output.push_str(&format!("seconds_since_epoch: {}\n", seconds_since_epoch));
        }
        write!(f, "{}", output)
    }
}
#[macro_export]
macro_rules! nonoptional_u32_val {
    ($val:expr) => {

        match $val.as_i64() {
            Some(s) => Ok(s.try_into().unwrap()),
            None => Err(ConversionError(concat!("Missing or invalid u32 for field: ", stringify!($val)))),
        }
    };
}
#[macro_export]
macro_rules! nonoptional_str_val {
    ($val:expr) => {
        match $val.as_str() {
            Some(s) => Ok(s.to_string()),
            None => Err(ConversionError(concat!("Missing or invalid string for field: ", stringify!($val)))),
        }
    };
}
#[macro_export]
macro_rules! uuid_val {
    ($val:expr) => {
        match $val.as_str() {
            None => None,
            Some(x) => Some(Uuid::parse_str(&x.to_string()).unwrap()),
        }
    };
}

#[macro_export]
macro_rules! str_val {
    ($val:expr) => {
        match $val.as_str() {
            None => None,
            Some(x) => Some(x.to_string()),
        }
    };
}

#[macro_export]
macro_rules! i64_val {
    ($val:expr) => {
        match $val.as_i64() {
            None => None,
            Some(x) => Some(x),
        }
    };
}

#[macro_export]
macro_rules! u32_val {
    ($val:expr) => {
        match $val.as_i64() {
            None => None,
            Some(x) => Some(x.try_into().unwrap()),
        }
    };
}

#[macro_export]
macro_rules! u16_val {
    ($val:expr) => {
        match $val.as_i64() {
            None => None,
            Some(x) => Some((x as u16).into()),
        }
    };
}

#[macro_export]
macro_rules! bool_val {
    ($val: expr) => {
        match $val.as_str().unwrap() {
            "true" => Some(true),
            "false" => Some(false),
            _ => Some(false),
        }
    };
}

#[derive(Clone, Debug)]
pub struct AgentID {
    pub agent_id: String,
}

#[derive(Clone, Debug)]
pub struct InventoryRootValues {
    pub folder_id: String,
}

#[derive(Clone, Debug)]
pub struct ClassifiedCategoriesValues {
    pub category_id: i32,
    pub category_name: String,
}

#[derive(Clone, Debug)]
pub struct UiConfig {
    pub allow_first_life: bool,
}

#[derive(Clone, Debug)]
pub struct LoginFlags {
    pub stipend_since_login: String, // stipend money recieved since last login
    pub ever_logged_in: bool,        // if the account has ever logged in
    pub seconds_since_epoch: Option<i64>, // server time in unix seconds since epoch
    // TODO: no longer sent by osgrid server
    pub daylight_savings: bool, // whether daylight savings is in effect for grid time
    pub gendered: bool,         // mysterious!
}

#[derive(Clone, Debug)]
pub struct GlobalTextures {
    pub cloud_texture_id: String,
    pub sun_texture_id: String,
    pub moon_texture_id: String,
}

#[derive(Clone, Debug)]
pub struct InitialOutfit {
    pub folder_name: String,
    pub gender: String,
}

#[derive(Clone, Debug)]
pub struct GesturesValues {
    pub item_id: String,  // the item ID of the gesture in the user's inventory
    pub asset_id: String, // the asset ID of the gesture
}

#[derive(Clone, Debug)]
pub struct BuddyListValues {
    pub buddy_id: String,                  //the UUID of the friend
    pub buddy_rights_given: FriendsRights, // the rights given to this user.
    pub buddy_rights_has: FriendsRights,
}

#[derive(Clone, Debug)]
pub struct FriendsRights {
    pub can_see_online: bool, // true if friend can see if you are online
    pub can_see_on_map: bool, // true if friend can see where you are on the map
    pub can_modify_objects: bool,
}

/// details about the child folders of the root folder
#[derive(Clone, Debug)]
pub struct InventorySkeletonValues {
    pub folder_id: String, // the ID of the folder
    pub parent_id: String, // the ID of the containing folder
    pub name: String,      // the name of the folder
    pub type_default: InventoryType,
    pub version: i32,
}

/// enum for agent access levels
#[derive(Clone, Debug, PartialEq)]
pub enum AgentAccess {
    Adult,
    Mature,
    PG,
    General,
}

/// Inventory item types
#[derive(Clone, Debug)]
pub enum InventoryType {
    Unknown,
    Texture,
    Sound,
    CallingCard,
    Landmark,
    Object,
    Notecard,
    Category,
    Folder,
    RootCategory,
    LSL,
    Snapshot,
    Attachment,
    Wearable,
    Animation,
    Gesture,
    Mesh,
}

/// The home location of the user. In the format
/// This is in the format "{'region_handle':[r<x-grid-coord>,r<y-grid-coord>],
///     'position':[r<x-region-coord>,r<y-region-coord>,r<z-region-coord>],
///     'look_at':[r<x-coord>,r<y-coord>,r<z-coord>]} in the XML
/// For example "{'region_handle':[r256000,r256000], 'position':[r50,r100,r200], 'look_at':[r1,r0,r0]}".
/// sent back to the client as a string instead of a struct for some reason :(
#[derive(Clone, Debug)]
pub struct HomeValues {
    pub region_handle: (String, String),
    pub position: (String, String, String),
    pub look_at: (String, String, String),
}

fn parse_agent_access(agent_access: Option<&xmlrpc::Value>) -> Option<AgentAccess> {
    agent_access.map(|x| match x.clone().as_str().unwrap() {
        "M" => AgentAccess::Mature,
        "A" => AgentAccess::Adult,
        "PG" => AgentAccess::PG,
        "G" => AgentAccess::General,
        _ => AgentAccess::General,
    })
}

fn parse_inventory_type(inventory_type: &xmlrpc::Value) -> InventoryType {
    match inventory_type.clone().as_i32().unwrap() {
        -1 => InventoryType::Unknown,
        0 => InventoryType::Texture,
        2 => InventoryType::Sound,
        3 => InventoryType::CallingCard,
        4 => InventoryType::Landmark,
        6 => InventoryType::Object,
        7 => InventoryType::Notecard,
        8 => InventoryType::Folder,
        9 => InventoryType::RootCategory,
        10 => InventoryType::LSL,
        15 => InventoryType::Snapshot,
        17 => InventoryType::Attachment,
        18 => InventoryType::Wearable,
        19 => InventoryType::Animation,
        20 => InventoryType::Gesture,
        22 => InventoryType::Mesh,
        _ => InventoryType::Unknown,
    }
}

fn string_3tuple(values: xmlrpc::Value) -> Option<(String, String, String)> {
    values.as_array().map(|x| {
        (
            x[0].as_str().unwrap().to_string(),
            x[1].as_str().unwrap().to_string(),
            x[2].as_str().unwrap().to_string(),
        )
    })
}

fn generate_friends_rights(rights: i32) -> FriendsRights {
    match rights {
        1 => FriendsRights {
            can_see_online: true,
            can_see_on_map: false,
            can_modify_objects: false,
        },
        2 => FriendsRights {
            can_see_online: true,
            can_see_on_map: true,
            can_modify_objects: false,
        },
        4 => FriendsRights {
            can_see_online: true,
            can_see_on_map: false,
            can_modify_objects: true,
        },
        _ => FriendsRights {
            can_see_online: false,
            can_see_on_map: false,
            can_modify_objects: false,
        },
    }
}

fn parse_buddy_list(values: Option<&xmlrpc::Value>) -> Option<Vec<BuddyListValues>> {
    let mut value_vec = vec![];
    let unwrapped_values = match values {
        None => return None,
        Some(x) => x.as_array().unwrap(),
    };
    for value in unwrapped_values {
        value_vec.push(BuddyListValues {
            buddy_id: value["buddy_id"].as_str().unwrap().to_string(),
            buddy_rights_given: generate_friends_rights(
                value["buddy_rights_given"].as_i32().unwrap(),
            ),
            buddy_rights_has: generate_friends_rights(value["buddy_rights_has"].as_i32().unwrap()),
        });
    }
    Some(value_vec)
}

fn parse_inventory_skeleton(
    values: Option<&xmlrpc::Value>,
) -> Option<Vec<InventorySkeletonValues>> {
    let mut value_vec = vec![];
    let unwrapped_values = match values {
        None => return None,
        Some(x) => x.as_array().unwrap(),
    };

    for value in unwrapped_values {
        value_vec.push(InventorySkeletonValues {
            folder_id: value["folder_id"].as_str().unwrap().to_string(),
            name: value["name"].as_str().unwrap().to_string(),
            parent_id: value["parent_id"].as_str().unwrap().to_string(),
            type_default: parse_inventory_type(&value["type_default"]),
            version: value["version"].as_i32().unwrap(),
        });
    }

    Some(value_vec)
}

fn parse_inventory_root(values: Option<&xmlrpc::Value>) -> Option<Vec<InventoryRootValues>> {
    let mut value_vec = vec![];
    let unwrapped_values = match values {
        None => return None,
        Some(x) => x.as_array().unwrap(),
    };
    for value in unwrapped_values {
        value_vec.push(InventoryRootValues {
            folder_id: value["folder_id"].as_str().unwrap().to_string(),
        });
    }
    Some(value_vec)
}

fn parse_inventory_lib_owner(values: Option<&xmlrpc::Value>) -> Option<Vec<AgentID>> {
    let mut value_vec = vec![];
    let unwrapped_values = match values {
        None => return None,
        Some(x) => x.as_array().unwrap(),
    };
    for value in unwrapped_values {
        value_vec.push(AgentID {
            agent_id: value["agent_id"].as_str().unwrap().to_string(),
        });
    }
    Some(value_vec)
}

fn parse_gestures(values: Option<&xmlrpc::Value>) -> Option<Vec<GesturesValues>> {
    let mut value_vec = vec![];
    let unwrapped_values = match values {
        None => return None,
        Some(x) => x.as_array().unwrap(),
    };
    for value in unwrapped_values {
        value_vec.push(GesturesValues {
            asset_id: value["asset_id"].as_str().unwrap().to_string(),
            item_id: value["item_id"].as_str().unwrap().to_string(),
        });
    }
    Some(value_vec)
}

fn parse_classified_categories(
    values: Option<&xmlrpc::Value>,
) -> Option<Vec<ClassifiedCategoriesValues>> {
    let mut value_vec = vec![];
    let unwrapped_values = match values {
        None => return None,
        Some(x) => x.as_array().unwrap(),
    };
    for value in unwrapped_values {
        value_vec.push(ClassifiedCategoriesValues {
            category_id: value["category_id"].as_i32().unwrap(),
            category_name: value["category_name"].as_str().unwrap().to_string(),
        });
    }
    Some(value_vec)
}

fn parse_initial_outfit(values: Option<&xmlrpc::Value>) -> Option<Vec<InitialOutfit>> {
    let mut value_vec = vec![];
    let unwrapped_values = match values {
        None => return None,
        Some(x) => x.as_array().unwrap(),
    };
    for value in unwrapped_values {
        value_vec.push(InitialOutfit {
            folder_name: value["folder_name"].as_str().unwrap().to_string(),
            gender: value["gender"].as_str().unwrap().to_string(),
        });
    }
    Some(value_vec)
}

fn parse_global_textures(values: Option<&xmlrpc::Value>) -> Option<Vec<GlobalTextures>> {
    let mut value_vec = vec![];
    let unwrapped_values = match values {
        None => return None,
        Some(x) => x.as_array().unwrap(),
    };
    for value in unwrapped_values {
        value_vec.push(GlobalTextures {
            cloud_texture_id: value["cloud_texture_id"].as_str().unwrap().to_string(),
            sun_texture_id: value["sun_texture_id"].as_str().unwrap().to_string(),
            moon_texture_id: value["moon_texture_id"].as_str().unwrap().to_string(),
        });
    }
    Some(value_vec)
}

fn parse_login_flags(values: Option<&xmlrpc::Value>) -> Option<Vec<LoginFlags>> {
    let mut value_vec = vec![];
    let unwrapped_values = match values {
        None => return None,
        Some(x) => x.as_array().unwrap(),
    };
    for value in unwrapped_values {
        value_vec.push(LoginFlags {
            stipend_since_login: value["stipend_since_login"].as_str().unwrap().to_string(),
            ever_logged_in: match value["ever_logged_in"].as_str().unwrap() {
                "Y" => true,
                "N" => false,
                _ => false,
            },
            seconds_since_epoch: value["seconds_since_epoch"].as_i64(),
            daylight_savings: match value["daylight_savings"].as_str().unwrap() {
                "Y" => true,
                "N" => false,
                _ => false,
            },
            gendered: match value["gendered"].as_str().unwrap() {
                "Y" => true,
                "N" => false,
                _ => false,
            },
        });
    }
    Some(value_vec)
}

fn parse_ui_config(values: Option<&xmlrpc::Value>) -> Option<Vec<UiConfig>> {
    let mut value_vec = vec![];
    let unwrapped_values = match values {
        None => return None,
        Some(x) => x.as_array().unwrap(),
    };
    for value in unwrapped_values {
        value_vec.push(UiConfig {
            allow_first_life: match value["allow_first_life"].as_str().unwrap() {
                "Y" => true,
                "N" => false,
                _ => false,
            },
        });
    }
    Some(value_vec)
}

// this is literally the worst thing I've ever been forced to do
// why this is the singular field that is passed as a string is beyond me
// this is so cursed and I hate it
//
impl From<xmlrpc::Value> for HomeValues {
    fn from(val: xmlrpc::Value) -> Self {
        let mut home_values_object = HomeValues {
            region_handle: ("".to_string(), "".to_string()),
            look_at: ("".to_string(), "".to_string(), "".to_string()),
            position: ("".to_string(), "".to_string(), "".to_string()),
        };

        // Convert xmlrpc::Value to string
        let valuestring = match val.as_str() {
            Some(s) => s.to_string(),
            None => return HomeValues {
                region_handle: ("Error".to_string(), "Invalid value".to_string()),
                look_at: ("Error".to_string(), "Invalid value".to_string(), "Invalid value".to_string()),
                position: ("Error".to_string(), "Invalid value".to_string(), "Invalid value".to_string()),
            },
        };

        // Split the string by "],"
        let split: Vec<&str> = valuestring.split("],").collect();

        for element in split {
            // Split by ":[" to separate labels from values
            let splitvalue: Vec<&str> = element.split(":[").collect();
            if splitvalue.len() != 2 {
                continue; // or handle the error as needed
            }

            let label = splitvalue[0].replace("{'", "").replace(['\'', ' '], "");
            let values = splitvalue[1].replace("]}", "");
            let values: Vec<&str> = values.split(',').collect();

            match label.as_str() {
                "region_handle" => {
                    if values.len() == 2 {
                        home_values_object.region_handle = (values[0].to_string(), values[1].to_string());
                    } else {
                        return HomeValues {
                            region_handle: ("Error".to_string(), "Invalid number of values".to_string()),
                            look_at: home_values_object.look_at,
                            position: home_values_object.position,
                        };
                    }
                }
                "look_at" => {
                    if values.len() == 3 {
                        home_values_object.look_at = (
                            values[0].to_string(),
                            values[1].to_string(),
                            values[2].to_string(),
                        );
                    } else {
                        return HomeValues {
                            region_handle: home_values_object.region_handle,
                            look_at: ("Error".to_string(), "Invalid number of values".to_string(), "Invalid value".to_string()),
                            position: home_values_object.position,
                        };
                    }
                }
                "position" => {
                    if values.len() == 3 {
                        home_values_object.position = (
                            values[0].to_string(),
                            values[1].to_string(),
                            values[2].to_string(),
                        );
                    } else {
                        return HomeValues {
                            region_handle: home_values_object.region_handle,
                            look_at: home_values_object.look_at,
                            position: ("Error".to_string(), "Invalid number of values".to_string(), "Invalid value".to_string()),
                        };
                    }
                }
                _ => continue,
            }
        }

        home_values_object
    }
}


#[derive(Debug)]
struct ConversionError(&'static str);

impl fmt::Display for ConversionError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Error for ConversionError {}

impl TryFrom<xmlrpc::Value> for LoginResponse {
    type Error = Box<dyn Error>;
    fn try_from(val: xmlrpc::Value) ->  Result<Self, Self::Error>  {
        Ok(LoginResponse {
            home: Some(val["home"].clone().into()),
            look_at: string_3tuple(val["look_at"].clone()),
            agent_access: parse_agent_access(val.get("agent_access")),
            agent_access_max: parse_agent_access(val.get("agent_access_max")),
            seed_capability: str_val!(val["seed_capability"]),
            first_name: nonoptional_str_val!(val["first_name"])?,
            last_name: nonoptional_str_val!(val["last_name"])?,
            agent_id: uuid_val!(val["agent_id"]),
            sim_ip: str_val!(val["sim_ip"]),
            sim_port: u16_val!(val["sim_port"]),
            http_port: u16_val!(val["http_port"]),
            start_location: str_val!(val["start_location"]),
            region_x: i64_val!(val["region_x"]),
            region_y: i64_val!(val["region_y"]),
            region_size_x: i64_val!(val["region_size_x"]),
            region_size_y: i64_val!(val["region_size_y"]),
            circuit_code: nonoptional_u32_val!(val["circuit_code"])?,
            session_id: uuid_val!(val["session_id"]),
            secure_session_id: uuid_val!(val["secure_session_id"]),
            inventory_root: parse_inventory_root(val.get("inventory-root")),
            inventory_skeleton: parse_inventory_skeleton(val.get("inventory-skeleton")),
            inventory_lib_root: parse_inventory_root(val.get("inventory-lib-root")),
            inventory_skeleton_lib: parse_inventory_skeleton(val.get("inventory-skel-lib")),
            inventory_lib_owner: parse_inventory_lib_owner(val.get("inventory-lib-owner")),
            map_server_url: str_val!(val["map-server-url"]),
            buddy_list: parse_buddy_list(val.get("buddy-list")),
            gestures: parse_gestures(val.get("gestures")),
            initial_outfit: parse_initial_outfit(val.get("initial-outfit")),
            global_textures: parse_global_textures(val.get("global-textures")),
            login: bool_val!(val["login"]),
            login_flags: parse_login_flags(val.get("login-flags")),
            message: str_val!(val["message"]),
            ui_config: parse_ui_config(val.get("ui-config")),
            event_categories: str_val!(val["event_categories"]),
            classified_categories: parse_classified_categories(val.get("classified_categories")),
            ..LoginResponse::default()
        })
    }
}
impl TryFrom<xmlrpc::Value> for LoginFailure {
    type Error = Box<dyn Error>;
    fn try_from(val: xmlrpc::Value) ->  Result<Self, Self::Error>  {
        Ok(LoginFailure {
            login: str_val!(val["login"]).ok_or(ConversionError("Missing login"))?,
            message: str_val!(val["message"]).ok_or(ConversionError("Missing message"))?,
            reason: str_val!(val["reason"]).ok_or(ConversionError("Missing reason"))?
        }
        )
    }
}

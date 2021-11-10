#[derive(Clone, Default)]
pub struct Session {
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
    pub first_name: Option<String>,      // The first name of the user
    pub last_name: Option<String>,       // The last name of the user
    pub agent_id: Option<String>,        // The id of the user
    pub sim_ip: Option<String>,          // The ip used to communicate with the recieving simulator
    pub sim_port: Option<i64>, // The UDP port used to communicate with the receiving simulator
    pub http_port: Option<i64>, // function unknown. Always set to 0 by OpenSimulator
    pub start_location: Option<String>, // The location where the user starts on login. "last", "home" or region location
    pub region_x: Option<i64>,          //The x grid coordinate of the start region in meters.
    // so a region at map co-ordinate 1000 will have a grid co-ordinate of 256000.
    pub region_y: Option<i64>, // the y grid coordinate of the start region in meters
    pub region_size_x: Option<i64>, // the size of the start region in meters.
    // usually will be 236 but with a varregion this can be a multiple
    // of 256
    pub region_size_y: Option<i64>,
    pub circuit_code: Option<i64>, // Circuit code to use for all UDP connections
    pub session_id: Option<String>, //UUID of this session
    pub secure_session_id: Option<String>, //the secure UUID of this session
    pub inventory_root: Option<InventoryRootValues>, // the ID of the user's root folder
    pub inventory_skeleton: Option<Vec<InventorySkeletonValues>>, // details about the child folders of the root folder.
    pub inventory_lib_root: Option<InventoryRootValues>, // the ID of the library root folder
    pub inventory_skeleton_lib: Option<Vec<InventorySkeletonValues>>, //details about the child folders of the library root folder
    pub inventory_lib_owner: Option<AgentID>, // the ID of the user that owns the library
    pub map_server_url: Option<String>,       //URL from which to request map tiles
    pub buddy_list: Option<Vec<BuddyListValues>>, // the user's friend list. Contains an entry for each friend
    pub gestures: Option<Vec<GesturesValues>>,    // the gestures the user currently has active.
    pub initial_outfit: Option<InitialOutfit>,    // use unknown, probably obsolete
    pub global_textures: Option<GlobalTextures>,  // use unknown, probably obsolete
    pub login: Option<bool>,                      // if logged in (should be true)
    pub login_flags: Option<LoginFlags>,
    pub message: Option<String>,
    pub ui_config: Option<UiConfig>,
    pub event_categories: Option<String>, // unknown
    pub classified_categories: Option<Vec<ClassifiedCategoriesValues>>,
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
macro_rules! bool_val {
    ($val: expr) => {
        match $val.as_str().unwrap() {
            "true" => Some(true),
            "false" => Some(false),
            _ => Some(false),
        }
    };
}
#[derive(Clone)]
pub struct AgentID {
    pub agent_id: String,
}

#[derive(Clone)]
pub struct InventoryRootValues {
    pub folder_id: String,
}

#[derive(Clone)]
pub struct ClassifiedCategoriesValues {
    pub category_id: i32,
    pub category_name: String,
}

#[derive(Clone)]
pub struct UiConfig {
    pub allow_first_life: bool,
}

#[derive(Clone)]
pub struct LoginFlags {
    pub stipend_since_login: String, // stipend money recieved since last login
    pub ever_logged_in: bool,        // if the account has ever logged in
    pub seconds_since_epoch: i64,    // server time in unix seconds since epoch
    pub daylight_savings: bool,      // whether daylight savings is in effect for grid time
    pub gendered: bool,              // mysterious!
}

#[derive(Clone)]
pub struct GlobalTextures {
    pub cloud_texture_id: String,
    pub sun_texture_id: String,
    pub moon_texture_id: String,
}

#[derive(Clone)]
pub struct InitialOutfit {
    pub folder_name: String,
    pub gender: String,
}

#[derive(Clone)]
pub struct GesturesValues {
    pub item_id: String,  // the item ID of the gesture in the user's inventory
    pub asset_id: String, // the asset ID of the gesture
}

#[derive(Clone)]
pub struct BuddyListValues {
    pub buddy_id: String,                  //the UUID of the friend
    pub buddy_rights_given: FriendsRights, // the rights given to this user.
    pub buddy_rights_has: FriendsRights,
}

#[derive(Clone)]
pub struct FriendsRights {
    pub can_see_online: bool, // true if friend can see if you are online
    pub can_see_on_map: bool, // true if friend can see where you are on the map
    pub can_modify_objects: bool,
}

/// details about the child folders of the root folder
#[derive(Clone)]
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
#[derive(Clone)]
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
#[derive(Clone)]
pub struct HomeValues {
    pub region_handle: (String, String),
    pub position: (String, String, String),
    pub look_at: (String, String, String),
}

fn parse_agent_access(agent_access: Option<&xmlrpc::Value>) -> Option<AgentAccess> {
    match agent_access {
        None => None,
        Some(x) => Some(match x.clone().as_str().unwrap() {
            "M" => AgentAccess::Mature,
            "A" => AgentAccess::Adult,
            "PG" => AgentAccess::PG,
            "G" => AgentAccess::General,
            _ => AgentAccess::General,
        }),
    }
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

fn string_2tuple(values: xmlrpc::Value) -> (String, String) {
    values.as_array().unwrap();
    (
        values[0].as_str().unwrap().to_string(),
        values[1].as_str().unwrap().to_string(),
    )
}

fn string_3tuple(values: xmlrpc::Value) -> (String, String, String) {
    values.as_array().unwrap();
    (
        values[0].as_str().unwrap().to_string(),
        values[1].as_str().unwrap().to_string(),
        values[2].as_str().unwrap().to_string(),
    )
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
    return Some(value_vec);
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

    return Some(value_vec);
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
    return Some(value_vec);
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
    return Some(value_vec);
}

impl Into<HomeValues> for xmlrpc::Value {
    fn into(self) -> HomeValues {
        HomeValues {
            region_handle: string_2tuple(self["region_handle"].clone()),
            look_at: string_3tuple(self["look_at"].clone()),
            position: string_3tuple(self["position"].clone()),
        }
    }
}

impl Into<InventoryRootValues> for xmlrpc::Value {
    fn into(self) -> InventoryRootValues {
        InventoryRootValues {
            folder_id: self["folder_id"].as_str().unwrap().to_string(),
        }
    }
}

impl Into<AgentID> for xmlrpc::Value {
    fn into(self) -> AgentID {
        AgentID {
            agent_id: self["agent_id"].as_str().unwrap().to_string(),
        }
    }
}

impl Into<InitialOutfit> for xmlrpc::Value {
    fn into(self) -> InitialOutfit {
        InitialOutfit {
            folder_name: self["folder_name"].as_str().unwrap().to_string(),
            gender: self["gender"].as_str().unwrap().to_string(),
        }
    }
}

impl Into<GlobalTextures> for xmlrpc::Value {
    fn into(self) -> GlobalTextures {
        GlobalTextures {
            cloud_texture_id: self["cloud_texture_id"].as_str().unwrap().to_string(),
            sun_texture_id: self["sun_texture_id"].as_str().unwrap().to_string(),
            moon_texture_id: self["moon_texture_id"].as_str().unwrap().to_string(),
        }
    }
}

impl Into<LoginFlags> for xmlrpc::Value {
    fn into(self) -> LoginFlags {
        LoginFlags {
            stipend_since_login: self["stipend_since_login"].as_str().unwrap().to_string(),
            ever_logged_in: match self["ever_logged_in"].as_str().unwrap() {
                "Y" => true,
                "N" => false,
                _ => false,
            },
            seconds_since_epoch: self["seconds_since_epoch"].as_i64().unwrap(),
            daylight_savings: match self["daylight_savings"].as_str().unwrap() {
                "Y" => true,
                "N" => false,
                _ => false,
            },
            gendered: match self["gendered"].as_str().unwrap() {
                "Y" => true,
                "N" => false,
                _ => false,
            },
        }
    }
}

impl Into<UiConfig> for xmlrpc::Value {
    fn into(self) -> UiConfig {
        UiConfig {
            allow_first_life: match self["allow_first_life"].as_str().unwrap() {
                "Y" => true,
                "N" => false,
                _ => false,
            },
        }
    }
}

impl Into<Session> for xmlrpc::Value {
    fn into(self) -> Session {
        Session {
            home: Some(self["home"].clone().into()),
            look_at: Some(string_3tuple(self["look_at"].clone())),
            agent_access: parse_agent_access(self.get("agent_access")),
            agent_access_max: parse_agent_access(self.get("agent_access_max")),
            seed_capability: str_val!(self["seed_capability"]),
            first_name: str_val!(self["first_name"]),
            last_name: str_val!(self["last_name"]),
            agent_id: str_val!(self["agent_id"]),
            sim_ip: str_val!(self["sim_ip"]),
            sim_port: i64_val!(self["sim_port"]),
            http_port: i64_val!(self["http_port"]),
            start_location: str_val!(self["start_location"]),
            region_x: i64_val!(self["region_x"]),
            region_y: i64_val!(self["region_y"]),
            region_size_x: i64_val!(self["region_size_x"]),
            region_size_y: i64_val!(self["region_size_y"]),
            circuit_code: i64_val!(self["circuit_code"]),
            session_id: str_val!(self["session_id"]),
            secure_session_id: str_val!(self["secure_session_id"]),
            inventory_root: Some(self["inventory-root"].clone().into()),
            inventory_skeleton: parse_inventory_skeleton(self.get("inventory-skeleton")),
            inventory_lib_root: Some(self["inventory-lib-root"].clone().into()),
            inventory_skeleton_lib: parse_inventory_skeleton(self.get("inventory-skel-lib")),
            inventory_lib_owner: Some(self["inventory-lib-owner"].clone().into()),
            map_server_url: str_val!(self["map-server-url"]),
            buddy_list: parse_buddy_list(self.get("buddy-list")),
            gestures: parse_gestures(self.get("gestures")),
            initial_outfit: Some(self["initial-outfit"].clone().into()),
            global_textures: Some(self["global-textures"].clone().into()),
            login: bool_val!(self["login"]),
            login_flags: Some(self["login-flags"].clone().into()),
            message: str_val!(self["message"]),
            ui_config: Some(self["ui-config"].clone().into()),
            event_categories: str_val!(self["event_categories"]),
            classified_categories: parse_classified_categories(self.get("classified_categories")),
            ..Session::default()
        }
    }
}

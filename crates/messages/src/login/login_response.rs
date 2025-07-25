/// This is cursed land.
/// the code in this file could be tried for treason.
/// this is for parsing absolutely busted xlm-rpc that comes out of the server, and should never be touched by anyone
/// unless it breaks.
/// I wrote this a long time ago, but it still works, and the less time you spend thinking about
/// xml-rpc as used in this project, the better.
use crate::{
    login::login_errors::ConversionError,
    utils::agent_access::{AgentAccess, parse_agent_access},
};
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::{collections::BTreeMap, str::FromStr};
use uuid::Uuid;
use xmlrpc_benthic::{self as xmlrpc, Value};

/// This is the full response struct of a successful opensimulator login.
/// you can find more information about it at <http://opensimulator.org/wiki/SimulatorLoginProtocol>
/// these types should be considered unstable until a 1.0.0 release of the login, due to the fact
/// that I may realize some of thse are not optional.
#[derive(Clone, Default, Debug, Serialize, Deserialize)]
pub struct LoginResponse {
    /// home: the home location of the user
    pub home: Option<HomeValues>,
    /// look_at: the direction the avatar should be facing
    /// This is a unit vector so
    /// (0, 1, 0) is facing straight north,
    /// (1, 0, 0) is east,
    /// (0,-1, 0) is south and
    /// (-1, 0, 0) is west.
    pub look_at: Option<(String, String, String)>,
    /// agent_access: The current maturity access level of the user
    pub agent_access: Option<AgentAccess>,
    /// agent_access_max: The maximum level of region the user can access
    pub agent_access_max: Option<AgentAccess>,
    /// The URL that the viewer should use to request further capabilities
    pub seed_capability: Option<String>,
    /// The first name of the user
    pub first_name: String,
    /// The last name of the user
    pub last_name: String,
    /// The id of the user
    pub agent_id: Uuid,
    /// The ip used to communicate with the receiving simulator
    pub sim_ip: Option<String>,
    /// The UDP port used to communicate with the receiving simulator
    pub sim_port: Option<u16>,
    /// Function unknown. Always set to 0 by OpenSimulator
    pub http_port: Option<u16>,
    /// The location where the user starts on login. "last", "home" or region location
    pub start_location: Option<String>,
    /// The x grid coordinate of the start region in meters.
    /// So a region at map coordinate 1000 will have a grid coordinate of 256000.
    pub region_x: Option<i64>,
    /// The y grid coordinate of the start region in meters
    pub region_y: Option<i64>,
    /// The size of the start region in meters.
    /// Usually will be 236 but with a varregion this can be a multiple of 256
    pub region_size_x: Option<i64>,
    /// Undocumented
    pub region_size_y: Option<i64>,
    /// Circuit code to use for all UDP connections
    pub circuit_code: u32,
    /// UUID of this session
    pub session_id: Uuid,
    /// The secure UUID of this session
    pub secure_session_id: Option<Uuid>,
    /// The ID of the user's root folder
    pub inventory_root: Option<Vec<Uuid>>,
    /// Details about the child folders of the root folder
    pub inventory_skeleton: Option<Vec<InventorySkeletonValues>>,
    /// The ID of the library root folder
    pub inventory_lib_root: Option<Vec<Uuid>>,
    /// Details about the child folders of the library root folder
    pub inventory_skeleton_lib: Option<Vec<InventorySkeletonValues>>,
    /// The ID of the user that owns the library
    pub inventory_lib_owner: Option<Vec<Uuid>>,
    /// URL from which to request map tiles
    pub map_server_url: Option<String>,
    /// The user's friend list. Contains an entry for each friend
    pub buddy_list: Option<Vec<BuddyListValues>>,
    /// The gestures the user currently has active
    pub gestures: Option<Vec<GesturesValues>>,
    /// Use unknown, probably obsolete
    pub initial_outfit: Option<Vec<InitialOutfit>>,
    /// Use unknown, probably obsolete
    pub global_textures: Option<Vec<GlobalTextures>>,
    /// If logged in (should be true)
    pub login: Option<bool>,
    /// Undocumented
    pub login_flags: Option<Vec<LoginFlags>>,
    /// Undocumented
    pub message: Option<String>,
    /// Undocumented
    pub ui_config: Option<Vec<UiConfig>>,
    /// Undocumented
    pub event_categories: Option<String>,
    /// Undocumented
    pub classified_categories: Option<Vec<ClassifiedCategoriesValues>>,
    /// Undocumented
    pub real_id: Option<String>,
    /// Undocumented
    pub search: Option<String>,
    /// Undocumented
    pub destination_guide_url: Option<String>,
    /// Undocumented
    pub event_notifications: Option<String>,
    /// Undocumented
    pub max_agent_groups: Option<i64>,
    /// Undocumented
    pub seconds_since_epoch: Option<i64>,
}
impl From<LoginResponse> for Value {
    fn from(val: LoginResponse) -> Self {
        let mut map = BTreeMap::new();

        if let Some(home) = val.home {
            map.insert("home".to_string(), home.into());
        }
        if let Some(look_at) = val.look_at {
            map.insert(
                "look_at".to_string(),
                Value::Array(vec![
                    Value::String(look_at.0),
                    Value::String(look_at.1),
                    Value::String(look_at.2),
                ]),
            );
        }
        if let Some(agent_access) = val.agent_access {
            map.insert("agent_access".to_string(), agent_access.into());
        }
        if let Some(agent_access_max) = val.agent_access_max {
            map.insert("agent_access_max".to_string(), agent_access_max.into());
        }
        if let Some(seed_capability) = val.seed_capability {
            map.insert(
                "seed_capability".to_string(),
                Value::String(seed_capability),
            );
        }

        map.insert("first_name".to_string(), Value::String(val.first_name));
        map.insert("last_name".to_string(), Value::String(val.last_name));

        map.insert(
            "agent_id".to_string(),
            Value::String(val.agent_id.to_string()),
        );
        if let Some(sim_ip) = val.sim_ip {
            map.insert("sim_ip".to_string(), Value::String(sim_ip));
        }
        if let Some(sim_port) = val.sim_port {
            map.insert("sim_port".to_string(), Value::Int(sim_port as i32));
        }
        if let Some(http_port) = val.http_port {
            map.insert("http_port".to_string(), Value::Int(http_port as i32));
        }
        if let Some(start_location) = val.start_location {
            map.insert("start_location".to_string(), Value::String(start_location));
        }
        if let Some(region_x) = val.region_x {
            map.insert("region_x".to_string(), Value::Int(region_x as i32));
        }
        if let Some(region_y) = val.region_y {
            map.insert("region_y".to_string(), Value::Int(region_y as i32));
        }
        if let Some(region_size_x) = val.region_size_x {
            map.insert(
                "region_size_x".to_string(),
                Value::Int(region_size_x as i32),
            );
        }
        if let Some(region_size_y) = val.region_size_y {
            map.insert(
                "region_size_y".to_string(),
                Value::Int(region_size_y as i32),
            );
        }

        map.insert(
            "circuit_code".to_string(),
            Value::Int(val.circuit_code as i32),
        );

        map.insert(
            "session_id".to_string(),
            Value::String(val.session_id.to_string()),
        );

        if let Some(secure_session_id) = val.secure_session_id {
            map.insert(
                "secure_session_id".to_string(),
                Value::String(secure_session_id.to_string()),
            );
        }

        if let Some(inventory_root) = val.inventory_root {
            map.insert(
                "inventory-root".to_string(),
                Value::Array(
                    inventory_root
                        .into_iter()
                        .map(|item| Value::String(item.to_string()))
                        .collect(),
                ),
            );
        }
        if let Some(inventory_skeleton) = val.inventory_skeleton {
            map.insert(
                "inventory-skeleton".to_string(),
                Value::Array(
                    inventory_skeleton
                        .into_iter()
                        .map(|item| item.into())
                        .collect(),
                ),
            );
        }
        if let Some(inventory_lib_root) = val.inventory_lib_root {
            map.insert(
                "inventory-lib-root".to_string(),
                Value::Array(
                    inventory_lib_root
                        .into_iter()
                        .map(|item| Value::String(item.to_string()))
                        .collect(),
                ),
            );
        }
        if let Some(inventory_skeleton_lib) = val.inventory_skeleton_lib {
            map.insert(
                "inventory-skel-lib".to_string(),
                Value::Array(
                    inventory_skeleton_lib
                        .into_iter()
                        .map(|item| item.into())
                        .collect(),
                ),
            );
        }
        if let Some(inventory_lib_owner) = val.inventory_lib_owner {
            map.insert(
                "inventory-lib-owner".to_string(),
                Value::Array(
                    inventory_lib_owner
                        .into_iter()
                        .map(|item| Value::String(item.to_string()))
                        .collect(),
                ),
            );
        }

        if let Some(map_server_url) = val.map_server_url {
            map.insert("map-server-url".to_string(), Value::String(map_server_url));
        }
        if let Some(buddy_list) = val.buddy_list {
            map.insert(
                "buddy-list".to_string(),
                Value::Array(buddy_list.into_iter().map(|item| item.into()).collect()),
            );
        }
        if let Some(gestures) = val.gestures {
            map.insert(
                "gestures".to_string(),
                Value::Array(gestures.into_iter().map(|item| item.into()).collect()),
            );
        }
        if let Some(initial_outfit) = val.initial_outfit {
            map.insert(
                "initial-outfit".to_string(),
                Value::Array(initial_outfit.into_iter().map(|item| item.into()).collect()),
            );
        }
        if let Some(global_textures) = val.global_textures {
            map.insert(
                "global-textures".to_string(),
                Value::Array(
                    global_textures
                        .into_iter()
                        .map(|item| item.into())
                        .collect(),
                ),
            );
        }
        if let Some(login) = val.login {
            map.insert("login".to_string(), Value::Bool(login));
        }
        if let Some(login_flags) = val.login_flags {
            map.insert(
                "login-flags".to_string(),
                Value::Array(login_flags.into_iter().map(|item| item.into()).collect()),
            );
        }
        if let Some(message) = val.message {
            map.insert("message".to_string(), Value::String(message));
        }
        if let Some(ui_config) = val.ui_config {
            map.insert(
                "ui-config".to_string(),
                Value::Array(ui_config.into_iter().map(|item| item.into()).collect()),
            );
        }
        if let Some(event_categories) = val.event_categories {
            map.insert(
                "event_categories".to_string(),
                Value::String(event_categories),
            );
        }
        if let Some(classified_categories) = val.classified_categories {
            map.insert(
                "classified_categories".to_string(),
                Value::Array(
                    classified_categories
                        .into_iter()
                        .map(|item| item.into())
                        .collect(),
                ),
            );
        }
        if let Some(real_id) = val.real_id {
            map.insert("real_id".to_string(), Value::String(real_id));
        }
        if let Some(search) = val.search {
            map.insert("search".to_string(), Value::String(search));
        }
        if let Some(destination_guide_url) = val.destination_guide_url {
            map.insert(
                "destination_guide_url".to_string(),
                Value::String(destination_guide_url),
            );
        }
        if let Some(event_notifications) = val.event_notifications {
            map.insert(
                "event_notifications".to_string(),
                Value::String(event_notifications),
            );
        }
        if let Some(max_agent_groups) = val.max_agent_groups {
            map.insert(
                "max_agent_groups".to_string(),
                Value::Int(max_agent_groups as i32),
            );
        }
        if let Some(seconds_since_epoch) = val.seconds_since_epoch {
            map.insert(
                "seconds_since_epoch".to_string(),
                Value::Int(seconds_since_epoch as i32),
            );
        }
        Value::Struct(map)
    }
}

#[macro_export]
/// extracts a u32 from an xmlrpc value, where the value cannot be None
macro_rules! nonoptional_u32_val {
    ($val:expr) => {
        match $val.as_i64() {
            Some(s) => Ok(s.try_into().unwrap()),
            None => Err(ConversionError(concat!(
                "Missing or invalid u32 for field: ",
                stringify!($val)
            ))),
        }
    };
}
#[macro_export]
/// extracts a str from an xmlrpc value, where the value cannot be None
macro_rules! nonoptional_str_val {
    ($val:expr) => {
        match $val.as_str() {
            Some(s) => Ok(s.to_string()),
            None => Err(ConversionError(concat!(
                "Missing or invalid string for field: ",
                stringify!($val)
            ))),
        }
    };
}

/// extracts a uuid from a xmlrpc value, where the value cannot be None
#[macro_export]
macro_rules! nonoptional_uuid_val {
    ($val:expr) => {
        match $val.as_str() {
            Some(s) => {
                Uuid::parse_str(s).expect(concat!("Invalid UUID in field: ", stringify!($val)))
            }
            None => panic!(concat!("Missing UUID field: ", stringify!($val))),
        }
    };
}

#[macro_export]
/// extracts a UUID from an xmlrpc value, where the value cannot be None
macro_rules! uuid_val {
    ($val:expr) => {
        match $val.as_str() {
            None => None,
            Some(x) => Some(Uuid::parse_str(&x.to_string()).unwrap()),
        }
    };
}

#[macro_export]
/// extracts a str from an xmlrpc value, where the value is an option
macro_rules! str_val {
    ($val:expr) => {
        match $val.as_str() {
            None => None,
            Some(x) => Some(x.to_string()),
        }
    };
}

#[macro_export]
/// extracts an i64 from an xmlrpc value, where the value is an option
macro_rules! i64_val {
    ($val:expr) => {
        match $val.as_i64() {
            None => None,
            Some(x) => Some(x),
        }
    };
}

#[macro_export]
/// extracts a u32 from an xmlrpc value, where the value is an option
macro_rules! u32_val {
    ($val:expr) => {
        match $val.as_i64() {
            None => None,
            Some(x) => Some(x.try_into().unwrap()),
        }
    };
}

#[macro_export]
/// extracts a u16 from an xmlrpc value, where the value is an option
macro_rules! u16_val {
    ($val:expr) => {
        match $val.as_i64() {
            None => None,
            Some(x) => Some((x as u16).into()),
        }
    };
}

#[macro_export]
/// extracts a bool from an xmlrpc value, where the value is an option
macro_rules! bool_val {
    ($val: expr) => {
        match $val.as_str().unwrap() {
            "true" => Some(true),
            "false" => Some(false),
            _ => Some(false),
        }
    };
}

#[derive(Clone, Debug, Serialize, Deserialize)]
/// Classified categories values. Used for storing the ID and name of classified categories.
pub struct ClassifiedCategoriesValues {
    /// The ID of the category
    pub category_id: i32,
    /// The name of the category
    pub category_name: String,
}
impl From<ClassifiedCategoriesValues> for Value {
    fn from(val: ClassifiedCategoriesValues) -> Self {
        let mut map = BTreeMap::new();
        map.insert("category_id".to_string(), Value::Int(val.category_id));
        map.insert(
            "category_name".to_string(),
            Value::String(val.category_name),
        );
        Value::Struct(map)
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
/// Struct for UI config. Not necessarily useful since it only contains a bool.
pub struct UiConfig {
    /// Allow for displaying "first life" field.
    pub allow_first_life: bool,
}
impl From<UiConfig> for Value {
    fn from(val: UiConfig) -> Self {
        let mut map = BTreeMap::new();
        map.insert(
            "allow_first_life".to_string(),
            Value::Bool(val.allow_first_life),
        );
        Value::Struct(map)
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
/// Flags sent from login. Currently unused.
pub struct LoginFlags {
    /// Stipend money received since last login
    pub stipend_since_login: String,
    /// If the account has ever logged in
    pub ever_logged_in: bool,
    /// Server time in Unix seconds since epoch
    pub seconds_since_epoch: Option<i64>,
    /// TODO: No longer sent by OSGrid server
    pub daylight_savings: bool,
    /// Mysterious!
    pub gendered: bool,
}
impl From<LoginFlags> for Value {
    fn from(val: LoginFlags) -> Self {
        let mut map = BTreeMap::new();
        map.insert(
            "stipend_since_login".to_string(),
            Value::String(val.stipend_since_login),
        );
        map.insert(
            "ever_logged_in".to_string(),
            Value::Bool(val.ever_logged_in),
        );
        if let Some(seconds) = val.seconds_since_epoch {
            map.insert(
                "seconds_since_epoch".to_string(),
                Value::Int(seconds as i32),
            );
        }
        map.insert(
            "daylight_savings".to_string(),
            Value::Bool(val.daylight_savings),
        );
        map.insert("gendered".to_string(), Value::Bool(val.gendered));
        Value::Struct(map)
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
/// Global textures for the region.
pub struct GlobalTextures {
    /// texture of clouds
    pub cloud_texture_id: String,
    /// texture of sun
    pub sun_texture_id: String,
    /// texture of moon
    pub moon_texture_id: String,
}
impl From<GlobalTextures> for Value {
    fn from(val: GlobalTextures) -> Self {
        let mut map = BTreeMap::new();
        map.insert(
            "cloud_texture_id".to_string(),
            Value::String(val.cloud_texture_id),
        );
        map.insert(
            "sun_texture_id".to_string(),
            Value::String(val.sun_texture_id),
        );
        map.insert(
            "moon_texture_id".to_string(),
            Value::String(val.moon_texture_id),
        );
        Value::Struct(map)
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
/// outfit the user is wearing when first logged in
pub struct InitialOutfit {
    /// the folder that contains the outfit
    pub folder_name: String,
    /// the gender of the outfit (not useful)
    pub gender: String,
}
impl From<InitialOutfit> for Value {
    fn from(val: InitialOutfit) -> Self {
        let mut map = BTreeMap::new();
        map.insert("folder_name".to_string(), Value::String(val.folder_name));
        map.insert("gender".to_string(), Value::String(val.gender));
        Value::Struct(map)
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
/// Gestures in the user's inventory
pub struct GesturesValues {
    /// the item ID of the gesture in the user's inventory
    pub item_id: String,
    /// the asset ID of the gesture
    pub asset_id: String,
}
impl From<GesturesValues> for Value {
    fn from(val: GesturesValues) -> Self {
        let mut map = BTreeMap::new();
        map.insert("item_id".to_string(), Value::String(val.item_id));
        map.insert("asset_id".to_string(), Value::String(val.asset_id));
        Value::Struct(map)
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
/// Values contained in the BuddyList, containing friends and rights
pub struct BuddyListValues {
    /// the UUID of the friend
    pub buddy_id: String,
    /// The rights that the friend has granted to this user.
    pub buddy_rights_given: FriendsRights,
    /// The rights that this user has granted to the friend.
    pub buddy_rights_has: FriendsRights,
}
impl From<BuddyListValues> for Value {
    fn from(val: BuddyListValues) -> Self {
        let mut map = BTreeMap::new();
        map.insert("buddy_id".to_string(), Value::String(val.buddy_id));
        map.insert(
            "buddy_rights_given".to_string(),
            val.buddy_rights_given.into(),
        );
        map.insert("buddy_rights_has".to_string(), val.buddy_rights_has.into());
        Value::Struct(map)
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
/// The struct containing the abilities friends can have
pub struct FriendsRights {
    /// true if friends can see if you are online
    pub can_see_online: bool,
    /// true if friends can see where you are on the map
    pub can_see_on_map: bool,
    /// true if friends can modify objects on the target avater
    pub can_modify_objects: bool,
}
impl From<FriendsRights> for Value {
    fn from(val: FriendsRights) -> Self {
        let mut map = BTreeMap::new();
        map.insert(
            "can_see_online".to_string(),
            Value::Bool(val.can_see_online),
        );
        map.insert(
            "can_see_on_map".to_string(),
            Value::Bool(val.can_see_on_map),
        );
        map.insert(
            "can_modify_objects".to_string(),
            Value::Bool(val.can_modify_objects),
        );
        Value::Struct(map)
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
/// details about the child folders of the root folder
pub struct InventorySkeletonValues {
    /// The ID of the folder
    pub folder_id: String,
    /// The ID of the containing folder
    pub parent_id: String,
    /// The name of the folder
    pub name: String,
    /// the default type of the object
    pub type_default: InventoryType,
    /// the version of the object
    pub version: i32,
}
impl From<InventorySkeletonValues> for Value {
    fn from(val: InventorySkeletonValues) -> Self {
        let mut map = BTreeMap::new();
        map.insert("folder_id".to_string(), Value::String(val.folder_id));
        map.insert("parent_id".to_string(), Value::String(val.parent_id));
        map.insert("name".to_string(), Value::String(val.name));
        map.insert("type_default".to_string(), val.type_default.into());
        map.insert("version".to_string(), Value::Int(val.version));
        Value::Struct(map)
    }
}
#[derive(Clone, Debug, Serialize, Deserialize)]
/// Inventory item types
pub enum InventoryType {
    /// Unknown object
    Unknown,
    /// Texture object
    Texture,
    /// Sound Object
    Sound,
    /// Calling cards are an object that allows you to see someone's profile, and view their
    /// online/offline status. Friend automatically receive each others calling cards.
    CallingCard,
    /// Landmark for teleporting to
    Landmark,
    /// 3d object
    Object,
    /// Notecard containing text
    Notecard,
    /// undocumented
    Category,
    /// A folder to contain inventory items within
    Folder,
    /// unknown
    RootCategory,
    /// integer. Unknown what this is used for.
    LSL,
    /// Screenshot
    Snapshot,
    /// An object that can be attached to a specific location, such as hand or head.
    Attachment,
    /// a wearable object
    Wearable,
    /// an animation
    Animation,
    /// an item that can trigger an animation, sound and emit text chat.
    Gesture,
    /// a 3d model, usually rigged that can be used as an avatar.
    Mesh,
}
impl From<InventoryType> for Value {
    fn from(val: InventoryType) -> Self {
        let value = match val {
            InventoryType::Unknown => -1,
            InventoryType::Texture => 0,
            InventoryType::Sound => 2,
            InventoryType::CallingCard => 3,
            InventoryType::Landmark => 4,
            InventoryType::Object => 6,
            InventoryType::Notecard => 7,
            InventoryType::Category => 8,
            InventoryType::Folder => 9,
            InventoryType::RootCategory => 10,
            InventoryType::LSL => 11,
            InventoryType::Snapshot => 15,
            InventoryType::Attachment => 17,
            InventoryType::Wearable => 18,
            InventoryType::Animation => 19,
            InventoryType::Gesture => 20,
            InventoryType::Mesh => 22,
        };
        Value::Int(value)
    }
}

/// The home location of the user. In the format
/// This is in the format `"{'region_handle':[r<x-grid-coord>,r<y-grid-coord>]`,
///     'position':`[r<x-region-coord>,r<y-region-coord>,r<z-region-coord>]`,
///     'look_at':`[r<x-coord>,r<y-coord>,r<z-coord>]`} in the XML
/// For example `"{'region_handle':[r256000,r256000], 'position':[r50,r100,r200], 'look_at':[r1,r0,r0]}"`.
/// sent back to the client as a string instead of a struct for some reason :(
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct HomeValues {
    /// x, y grid coordinate of the home location
    pub region_handle: (String, String),
    /// x y z position of the user in the home location
    pub position: (String, String, String),
    /// x y z location the user is looking
    pub look_at: (String, String, String),
}
impl From<HomeValues> for Value {
    fn from(val: HomeValues) -> Self {
        let mut map = BTreeMap::new();
        map.insert(
            "region_handle".to_string(),
            Value::Array(vec![
                Value::String(val.region_handle.0),
                Value::String(val.region_handle.1),
            ]),
        );
        map.insert(
            "position".to_string(),
            Value::Array(vec![
                Value::String(val.position.0),
                Value::String(val.position.1),
                Value::String(val.position.2),
            ]),
        );
        map.insert(
            "look_at".to_string(),
            Value::Array(vec![
                Value::String(val.look_at.0),
                Value::String(val.look_at.1),
                Value::String(val.look_at.2),
            ]),
        );
        Value::Struct(map)
    }
}

// converts xmlrpc to inventory type enum
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

/// converts xmlrpc to string_3tuples
fn string_3tuple(values: xmlrpc::Value) -> Option<(String, String, String)> {
    values.as_array().map(|x| {
        (
            x[0].as_str().unwrap().to_string(),
            x[1].as_str().unwrap().to_string(),
            x[2].as_str().unwrap().to_string(),
        )
    })
}

/// converts friendRights int values to a FriendsRights struct
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

/// converts xlmrpc to a buddy list
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

/// converts xmlrpc to an inventory_skeelton object
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

/// converts xmlrpc to a UUID
fn parse_inventory_root(values: Option<&xmlrpc::Value>) -> Option<Vec<Uuid>> {
    let mut value_vec = vec![];
    let unwrapped_values = match values {
        None => return None,
        Some(x) => x.as_array().unwrap(),
    };
    for value in unwrapped_values {
        if let Ok(id) = Uuid::from_str(value["folder_id"].as_str().unwrap()) {
            {
                value_vec.push(id);
            }
        }
    }
    Some(value_vec)
}

/// converts xmlrpc to an inventory_lib_owner object
fn parse_inventory_lib_owner(values: Option<&xmlrpc::Value>) -> Option<Vec<Uuid>> {
    let mut value_vec = vec![];
    let unwrapped_values = match values {
        None => return None,
        Some(x) => x.as_array().unwrap(),
    };
    for value in unwrapped_values {
        if let Ok(id) = Uuid::from_str(value["agent_id"].as_str().unwrap()) {
            {
                value_vec.push(id);
            }
        }
    }
    Some(value_vec)
}

/// converts xmlrpc to a gesturesvalues object
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
/// converts an xmlrpc to a classified catoegories values object  
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

/// converts xmlrpc to an IntiialOutfit object
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

/// converts xmlrpc to a GlobalTextures object  
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

/// converts xmlrpc to LoginFlags
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

/// converts xlmrpc to UiConfig
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

/// converts xmlrpc to a HomeValues object
// this is literally the worst thing I've ever been forced to do
// why this is the singular field that is passed as a string is beyond me
// this is so cursed and I hate it
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
            None => {
                return HomeValues {
                    region_handle: ("Error".to_string(), "Invalid value".to_string()),
                    look_at: (
                        "Error".to_string(),
                        "Invalid value".to_string(),
                        "Invalid value".to_string(),
                    ),
                    position: (
                        "Error".to_string(),
                        "Invalid value".to_string(),
                        "Invalid value".to_string(),
                    ),
                };
            }
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
                        home_values_object.region_handle =
                            (values[0].to_string(), values[1].to_string());
                    } else {
                        return HomeValues {
                            region_handle: (
                                "Error".to_string(),
                                "Invalid number of values".to_string(),
                            ),
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
                            look_at: (
                                "Error".to_string(),
                                "Invalid number of values".to_string(),
                                "Invalid value".to_string(),
                            ),
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
                            position: (
                                "Error".to_string(),
                                "Invalid number of values".to_string(),
                                "Invalid value".to_string(),
                            ),
                        };
                    }
                }
                _ => continue,
            }
        }

        home_values_object
    }
}

/// converts from xlmrpc to a LoginResponse
impl TryFrom<xmlrpc::Value> for LoginResponse {
    type Error = Box<dyn Error>;
    fn try_from(val: xmlrpc::Value) -> Result<Self, Self::Error> {
        Ok(LoginResponse {
            home: Some(val["home"].clone().into()),
            look_at: string_3tuple(val["look_at"].clone()),
            agent_access: parse_agent_access(val.get("agent_access")),
            agent_access_max: parse_agent_access(val.get("agent_access_max")),
            seed_capability: str_val!(val["seed_capability"]),
            first_name: nonoptional_str_val!(val["first_name"])?,
            last_name: nonoptional_str_val!(val["last_name"])?,
            agent_id: nonoptional_uuid_val!(val["agent_id"]),
            sim_ip: str_val!(val["sim_ip"]),
            sim_port: u16_val!(val["sim_port"]),
            http_port: u16_val!(val["http_port"]),
            start_location: str_val!(val["start_location"]),
            region_x: i64_val!(val["region_x"]),
            region_y: i64_val!(val["region_y"]),
            region_size_x: i64_val!(val["region_size_x"]),
            region_size_y: i64_val!(val["region_size_y"]),
            circuit_code: nonoptional_u32_val!(val["circuit_code"])?,
            session_id: nonoptional_uuid_val!(val["session_id"]),
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

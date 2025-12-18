use crate::{
    errors::ParseError, http::login::login_error::LoginError, utils::agent_access::AgentAccess,
};
use glam::{Vec2, Vec3};
use serde::{Deserialize, Serialize};
use serde_llsd_benthic::converter::{get, get_nested_vec, get_opt, get_vec, FromLLSDValue};
use serde_llsd_benthic::{auto_from_str, LLSDValue};
use std::collections::HashMap;
use uuid::Uuid;

/// Represents the result of a login attempt parsed from a valid LLSD XML response.
pub enum LoginStatus {
    /// Login succeeded and contains a valid LoginResponse
    Success(Box<LoginResponse>),
    /// Login failed, and contains error information
    Failure(LoginError),
}

/// This is the full response struct of a successful opensimulator login.
/// you can find more information about it at <http://opensimulator.org/wiki/SimulatorLoginProtocol>
#[derive(Clone, Default, Debug, Serialize, Deserialize)]
pub struct LoginResponse {
    /// The first name of the user
    pub first_name: String,
    /// The last name of the user
    pub last_name: String,
    /// The id of the user
    pub agent_id: Uuid,
    /// UUID of this session
    pub session_id: Uuid,
    /// The ip used to communicate with the receiving simulator
    pub sim_ip: String,
    /// The UDP port used to communicate with the receiving simulator
    pub sim_port: u16,
    /// The URL that the viewer should use to request further capabilities
    pub seed_capability: Option<String>,
    /// The ID of the user's root folder
    pub inventory_root: Option<Uuid>,
    /// Details about the child folders of the root folder
    pub inventory_skeleton: Option<Vec<InventorySkeletonValues>>,
    /// The ID of the library root folder
    pub inventory_lib_root: Option<Uuid>,
    /// Details about the child folders of the library root folder
    pub inventory_skeleton_lib: Option<Vec<InventorySkeletonValues>>,
    /// The ID of the user that owns the library
    pub inventory_lib_owner: Option<Uuid>,
    /// home: the home location of the user
    /// TODO: should not be optional
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
    /// The secure UUID of this session
    pub secure_session_id: Option<Uuid>,
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

impl LoginResponse {
    /// Parses an LLSD-formatted XML login response into a `LoginResponse` struct.
    /// If the login failed, returns a `LoginError` with details from the response.
    pub fn from_xmlrpc(data: &str) -> Result<LoginStatus, ParseError> {
        match auto_from_str(data) {
            Ok(xml) => match xml {
                LLSDValue::Map(ref map) => {
                    // Login is false when the login fails, and true when it succeeds.
                    // If the login response fails, return a loginError instead
                    let login: bool = get("login", map);
                    if !login {
                        return Ok(LoginStatus::Failure(LoginError::from_llsd(&xml)));
                    }

                    Ok(LoginStatus::Success(Box::new(LoginResponse {
                        first_name: get("first_name", map),
                        last_name: get("last_name", map),
                        agent_id: get("agent_id", map),
                        session_id: get("session_id", map),
                        sim_ip: get("sim_ip", map),
                        sim_port: get("sim_port", map),
                        circuit_code: get("circuit_code", map),
                        seed_capability: get_opt("seed_capability", map),
                        http_port: get_opt("http_port", map),
                        login: get_opt("login", map),
                        message: get_opt("message", map),
                        ui_config: get_nested_vec("ui_config", map),
                        event_notifications: get_opt("event_notifications", map),
                        inventory_root: get_inventory_root(map),
                        inventory_lib_root: get_nested_value("inventory-lib-root", map),
                        inventory_lib_owner: get_inventory_lib_owner(map),
                        inventory_skeleton: get_nested_vec("inventory-skeleton", map),
                        inventory_skeleton_lib: get_nested_vec("inventory-skel-lib", map),
                        classified_categories: get_nested_vec("classified_categories", map),
                        initial_outfit: get_nested_vec("initial-outfit", map),
                        region_x: get_opt("region_x", map),
                        region_y: get_opt("region_y", map),
                        start_location: get_opt("start_location", map),
                        event_categories: get_opt("event_categories", map),
                        buddy_list: get_vec("buddy_list", map),
                        region_size_x: get_opt("region_size_x", map),
                        region_size_y: get_opt("region_size_y", map),
                        gestures: get_nested_vec("gestures", map),
                        seconds_since_epoch: get_opt("seconds_since_epoch", map),
                        login_flags: get_nested_vec("login-flags", map),
                        map_server_url: get_opt("map_server_url", map),
                        agent_access_max: get_opt("agent_access_max", map),
                        secure_session_id: get_opt("secure_session_id", map),
                        home: get_opt("home", map),
                        global_textures: get_nested_vec("global_textures", map),
                        ..Default::default()
                    })))
                }
                err => Err(err)?,
            },
            Err(e) => Err(e)?,
        }
    }
}

fn get_inventory_root(map: &HashMap<String, LLSDValue>) -> Option<Uuid> {
    map.get("inventory-root").and_then(|v| {
        if let LLSDValue::Array(arr) = v {
            arr.iter().find_map(|item| {
                if let LLSDValue::Map(inner_map) = item {
                    get_opt::<Uuid>("folder_id", inner_map)
                } else {
                    None
                }
            })
        } else {
            None
        }
    })
}

/// retrieve inventory lib owner from llsd
pub fn get_inventory_lib_owner(map: &HashMap<String, LLSDValue>) -> Option<Uuid> {
    map.get("inventory-lib-owner").and_then(|v| {
        if let LLSDValue::Array(arr) = v {
            for item in arr {
                if let LLSDValue::Map(inner_map) = item {
                    // each inner_map represents the <struct> with agent_id
                    if let Some(uuid) = get_opt::<Uuid>("agent_id", inner_map) {
                        return Some(uuid); // return first agent_id found
                    }
                }
            }
        }
        None
    })
}

/// retrieve nested value from llsd
pub fn get_nested_value<T: FromLLSDValue>(
    key: &str,
    map: &HashMap<String, LLSDValue>,
) -> Option<T> {
    map.get(key).and_then(|v| {
        if let LLSDValue::Array(arr) = v {
            for inner in arr {
                if let LLSDValue::Array(inner_arr) = inner {
                    for item in inner_arr {
                        if let Some(parsed) = T::from_llsd(item) {
                            return Some(parsed); // return the first parsed value
                        }
                    }
                }
            }
        }
        None
    })
}
#[derive(Clone, Debug, Serialize, Deserialize)]
/// Classified categories values. Used for storing the ID and name of classified categories.
pub struct ClassifiedCategoriesValues {
    /// The ID of the category
    pub category_id: i32,
    /// The name of the category
    pub category_name: String,
}
impl FromLLSDValue for ClassifiedCategoriesValues {
    fn from_llsd(value: &LLSDValue) -> Option<Self> {
        if let LLSDValue::Map(map) = value {
            Some(ClassifiedCategoriesValues {
                category_id: get("category_id", map),
                category_name: get("category_name", map),
            })
        } else {
            None
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
/// Struct for UI config. Not necessarily useful since it only contains a bool.
pub struct UiConfig {
    /// Allow for displaying "first life" field.
    pub allow_first_life: bool,
}
impl FromLLSDValue for UiConfig {
    fn from_llsd(value: &LLSDValue) -> Option<Self> {
        if let LLSDValue::Map(map) = value {
            Some(UiConfig {
                allow_first_life: get("allow_first_life", map),
            })
        } else {
            None
        }
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
impl FromLLSDValue for LoginFlags {
    fn from_llsd(value: &LLSDValue) -> Option<Self> {
        if let LLSDValue::Map(map) = value {
            Some(LoginFlags {
                stipend_since_login: get("stipend_since_login", map),
                ever_logged_in: get("ever_logged_in", map),
                seconds_since_epoch: map.get("seconds_since_epoch").and_then(|v| {
                    if let LLSDValue::Integer(i) = v {
                        Some(*i as i64)
                    } else if let LLSDValue::Undefined = v {
                        None
                    } else {
                        None
                    }
                }),
                daylight_savings: get("daylight_savings", map),
                gendered: get("gendered", map),
            })
        } else {
            None
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
/// Global textures for the region.
pub struct GlobalTextures {
    /// texture of clouds
    pub cloud_texture_id: Uuid,
    /// texture of sun
    pub sun_texture_id: Uuid,
    /// texture of moon
    pub moon_texture_id: Uuid,
}
impl FromLLSDValue for GlobalTextures {
    fn from_llsd(value: &LLSDValue) -> Option<Self> {
        if let LLSDValue::Map(map) = value {
            Some(GlobalTextures {
                cloud_texture_id: get("cloud_texture_id", map),
                sun_texture_id: get("sun_texture_id", map),
                moon_texture_id: get("moon_texture_id", map),
            })
        } else {
            None
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
/// outfit the user is wearing when first logged in
pub struct InitialOutfit {
    /// the folder that contains the outfit
    pub folder_name: Uuid,
    /// the gender of the outfit (not useful)
    pub gender: String,
}
impl FromLLSDValue for InitialOutfit {
    fn from_llsd(value: &LLSDValue) -> Option<Self> {
        if let LLSDValue::Map(map) = value {
            Some(InitialOutfit {
                folder_name: get("folder_name", map),
                gender: get("gender", map),
            })
        } else {
            None
        }
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
impl FromLLSDValue for GesturesValues {
    fn from_llsd(value: &LLSDValue) -> Option<Self> {
        if let LLSDValue::Map(map) = value {
            Some(GesturesValues {
                item_id: get("item_id", map),
                asset_id: get("asset_id", map),
            })
        } else {
            None
        }
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
impl FromLLSDValue for BuddyListValues {
    fn from_llsd(value: &LLSDValue) -> Option<Self> {
        if let LLSDValue::Map(map) = value {
            Some(BuddyListValues {
                buddy_id: get("buddy_id", map),
                buddy_rights_given: get("buddy_rights_given", map),
                buddy_rights_has: get("buddy_rights_has", map),
            })
        } else {
            None
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
/// The struct containing the abilities friends can have
pub struct FriendsRights {
    /// true if friends can see if you are online
    pub can_see_online: bool,
    /// true if friends can see where you are on the map
    pub can_see_on_map: bool,
    /// true if friends can modify objects on the target avater
    pub can_modify_objects: bool,
}
impl FriendsRights {
    fn _from_int(rights: u8) -> Self {
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
}
impl FromLLSDValue for FriendsRights {
    fn from_llsd(value: &LLSDValue) -> Option<Self> {
        if let LLSDValue::Map(map) = value {
            Some(FriendsRights {
                can_see_online: get("can_see_online", map),
                can_see_on_map: get("can_see_on_map", map),
                can_modify_objects: get("can_modify_objects", map),
            })
        } else {
            None
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
/// details about the child folders of the root folder
pub struct InventorySkeletonValues {
    /// The ID of the folder
    pub folder_id: Uuid,
    /// The ID of the containing folder
    pub parent_id: Uuid,
    /// The name of the folder
    pub name: String,
    /// the default type of the object
    pub type_default: InventoryType,
    /// the version of the object
    pub version: i32,
}
impl FromLLSDValue for InventorySkeletonValues {
    fn from_llsd(value: &LLSDValue) -> Option<Self> {
        if let LLSDValue::Map(map) = value {
            Some(InventorySkeletonValues {
                folder_id: get("folder_id", map),
                parent_id: get("parent_id", map),
                name: get("name", map),
                type_default: get("type_default", map),
                version: get("version", map),
            })
        } else {
            None
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
/// Inventory item types
pub enum InventoryType {
    /// Unknown object
    #[default]
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

impl FromLLSDValue for InventoryType {
    fn from_llsd(value: &LLSDValue) -> Option<Self> {
        if let LLSDValue::Integer(i) = value {
            match *i {
                -1 => Some(InventoryType::Unknown),
                0 => Some(InventoryType::Texture),
                2 => Some(InventoryType::Sound),
                3 => Some(InventoryType::CallingCard),
                4 => Some(InventoryType::Landmark),
                6 => Some(InventoryType::Object),
                7 => Some(InventoryType::Notecard),
                8 => Some(InventoryType::Category),
                9 => Some(InventoryType::Folder),
                10 => Some(InventoryType::RootCategory),
                11 => Some(InventoryType::LSL),
                15 => Some(InventoryType::Snapshot),
                17 => Some(InventoryType::Attachment),
                18 => Some(InventoryType::Wearable),
                19 => Some(InventoryType::Animation),
                20 => Some(InventoryType::Gesture),
                22 => Some(InventoryType::Mesh),
                _ => None, // unknown integer
            }
        } else {
            None
        }
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
    pub region_handle: Vec2,
    /// x y z position of the user in the home location
    pub position: Vec3,
    /// x y z location the user is looking
    pub look_at: Vec3,
}
impl FromLLSDValue for HomeValues {
    fn from_llsd(value: &LLSDValue) -> Option<Self> {
        if let LLSDValue::String(s) = value {
            // Trim braces and whitespace
            let s = s.trim_matches(|c| c == '{' || c == '}').trim();

            // Remove field names and brackets, keep only numbers with 'r' prefix
            let s = s
                .replace("region_handle", "")
                .replace("position", "")
                .replace("look_at", "")
                .replace("[", "")
                .replace("]", "")
                .replace("'", "")
                .replace(" ", "");

            // Split by commas
            let parts: Vec<&str> = s.split(',').collect();

            // Expect 2 + 3 + 3 numbers
            if parts.len() != 8 {
                eprintln!(
                    "Unexpected home string format, got {} parts: {:?}",
                    parts.len(),
                    parts
                );
                return None;
            }

            // Parse numbers, stripping any leading 'r' or ':' characters
            let nums: Vec<f32> = parts
                .iter()
                .map(|p| p.trim_start_matches(|c| c == 'r' || c == ':'))
                .map(|p| p.parse::<f32>().ok())
                .collect::<Option<Vec<f32>>>()?;

            let home = HomeValues {
                region_handle: Vec2::new(nums[0], nums[1]),
                position: Vec3::new(nums[2], nums[3], nums[4]),
                look_at: Vec3::new(nums[5], nums[6], nums[7]),
            };
            Some(home)
        } else {
            None
        }
    }
}

#[derive(Clone, Default)]
pub struct Session {
    pub home: Option<HomeValues>,         // the home location of the user
    pub look_at: Option<(i64, i64, i64)>, // the direction the avatar should be facing
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
    pub sim_port: Option<String>, // The UDP port used to communicate with the receiving simulator
    pub http_port: Option<String>, // function unknown. Always set to 0 by OpenSimulator
    pub start_location: Option<String>, // The location where the user starts on login. "last", "home" or region location
    pub region_x: Option<i64>,          //The x grid coordinate of the start region in meters.
    // so a region at map co-ordinate 1000 will have a grid co-ordinate of 256000.
    pub region_y: Option<i64>, // the y grid coordinate of the start region in meters
    pub region_size_x: Option<i64>, // the size of the start region in meters.
    // usually will be 236 but with a varregion this can be a multiple
    // of 256
    pub circuit_code: Option<String>, // Circuit code to use for all UDP connections
    pub session_id: Option<String>,   //UUID of this session
    pub secure_session_id: Option<String>, //the secure UUID of this session
    pub inventory_root: Option<String>, // the ID of the user's root folder
    pub inventory_skeleton: Option<Vec<InvSkelValues>>, // details about the child folders of the root folder.
    pub inventory_lib_root: Option<String>,             // the ID of the library root folder
    pub inventory_skel_lib: Option<Vec<InvSkelValues>>, //details about the child folders of the library root folder
    pub inventory_lib_owner: Option<String>,            // the ID of the user that owns the library
    pub map_server_url: Option<String>,                 //URL from which to request map tiles
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

#[derive(Clone)]
pub struct ClassifiedCategoriesValues {
    pub id: i32,
    pub name: String,
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
    pub buddy_id: String,                       //the UUID of the friend
    pub buddy_rights_given: Vec<FriendsRights>, // the rights given to this user.
    pub buddy_rights_has: Vec<FriendsRights>,
}

#[derive(Clone)]
pub struct FriendsRights {
    pub uuid: String,                   //system ID of the avatar
    pub name: String,                   // full name of the avatar
    pub is_online: bool,                // true if avatar is online
    pub can_see_me_online: bool,        // true if friend can see if you are online
    pub can_see_on_map: bool,           // true if friend can see where you are on the map
    pub can_modify_my_objects: bool,    // true if friend can modify your objects
    pub can_see_them_online: bool,      // true if you can see friend online
    pub can_see_them_on_map: bool,      // true if you can see friend on map
    pub can_modify_their_objects: bool, // true if you can modify their objects
}

/// details about the child folders of the root folder
#[derive(Clone)]
pub struct InvSkelValues {
    pub folder_id: String, // the ID of the folder
    pub parent_id: String, // the ID of the containing folder
    pub name: String,      // the name of the folder
    pub type_default: InventoryType,
    pub version: String,
}

/// enum for agent access levels
#[derive(Clone)]
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
    pub region_handle: (i64, i64),
    pub position: (i64, i64, i64),
    pub look_at: (i64, i64, i64),
}

fn parse_agent_access(agent_access: Option<&xmlrpc::Value>) -> Option<AgentAccess> {
    match agent_access {
        None => None,
        Some(x) => {
            if x.clone() == xmlrpc::Value::from("M") {
                Some(AgentAccess::Mature)
            } else if x.clone() == xmlrpc::Value::from("A") {
                Some(AgentAccess::Adult)
            } else if x.clone() == xmlrpc::Value::from("PG") {
                Some(AgentAccess::PG)
            } else if x.clone() == xmlrpc::Value::from("G") {
                Some(AgentAccess::General)
            } else {
                Some(AgentAccess::General)
            }
        }
    }
}

impl Into<Session> for xmlrpc::Value {
    fn into(self) -> Session {
        let home_values: HomeValues = HomeValues {
            region_handle: (1, 2),
            look_at: (1, 2, 3),
            position: (1, 2, 3),
        };
        Session {
            home: Some(home_values),
            // TODO: make this work lmao
            first_name: Some(self["first"].as_str().unwrap().to_string()),
            agent_access: parse_agent_access(self.get("agent_access")),
            agent_access_max: parse_agent_access(self.get("agent_access_max")),
            ..Session::default()
        }
    }
}

use std::{
    collections::HashMap,
    fs::{File, create_dir_all},
    io::{self, Write},
    path::{Path, PathBuf},
    sync::Arc,
};

use glam::{U16Vec2, Vec2};
use serde::Serialize;
use sqlx::{Pool, Sqlite};
use uuid::Uuid;

use crate::errors::SessionError;

/// Message and struct for the current user's session.
///
/// This includes all data that will be used throughout the session, and much of it is populated by
/// the LoginResponse packet. This sets the active session for the Mailbox, and ensures the UDP
/// socket doesn't close.
///
/// # Cause
/// - A login is triggered by the UI, and the handle_login function is called
///
/// # Effects
/// - Starts UDP read between client and server
#[derive(Debug)]
pub struct Session<Capability, Avatar, Land, UdpSocket> {
    /// address of the server the client is connected to. formatted http://Url:Socket
    pub address: String,
    /// agent ID of the user
    pub agent_id: Uuid,
    /// session ID of the user
    pub session_id: Uuid,
    /// the running UDP socket attached to the session  
    pub socket: Option<Arc<UdpSocket>>,
    /// The sequence number of the packets sent. Created as a simple count from the core to the server.
    pub sequence_number: u16,
    /// The local IP that the login is sent from. This is stored to ensure the IP of the
    /// UseCircuitCode packet is sent from the same IP as the login, to prevent server errors.
    pub local_ip: std::net::IpAddr,
    /// The URL endpoint to request more capabilities
    pub seed_capability_url: String,
    /// The HashMap for storing capability URLs
    pub capability_urls: HashMap<Capability, String>,
    /// inventory details retrieved from initial login
    pub inventory_data: InventoryData,
    /// The environment cache. Contains things for handling and generating the environment.
    pub environment_cache: EnvironmentCache<Land>,
    /// The agent list. Contains information about the appearances of all loaded agents
    pub avatars: HashMap<Uuid, Avatar>,
    /// data about the region the user is currently in
    pub region_data: RegionData,
    /// the connection to the inventory sqlite DB
    /// this stores folder data and inventory metadata
    pub inventory_db_connection: Pool<Sqlite>,
}

#[derive(Debug, Default)]
/// Information about the current region the user is in
pub struct RegionData {
    /// The region global height of the water. This is used to render a flat plane of water over
    /// the entire region.
    pub water_height: f32,
    /// The time elapsed since there was an update for the region's time
    pub last_time_update: u64,
    /// Coordinates of the region in the world. Region x and region y from the login response.
    pub region_coordinates: Vec2,
    /// ID of the region. This is a combination of the sim IP and port.
    pub region_id: String,
}

/// The state of the Mailbox, if it is running, starting, stopping or stopped.
#[derive(Debug, Clone, PartialEq)]
pub enum ServerState {
    /// The mailbox starts in the Starting state
    Starting,
    /// The mailbox is running
    Running,
    /// The mailbox is preparing to stop
    Stopping,
    /// the mailbox is stopped
    Stopped,
}

/// Contains information about the Inventory
#[derive(Debug)]
pub struct InventoryData {
    /// The root of the inventory, received from the LoginResponse. This is a vector of the base
    /// UUIDs that will be used to create the root of the inventory tree using a
    /// FetchInventoryDescendents2 call.
    pub inventory_root: Uuid,
    /// The UUID of the owner of the inventory lib. Used to create the FetchLibDescendents2 call.
    pub inventory_lib_owner: Uuid,
    /// boolean to signify the inventory has successfully loaded and is ready for use.
    pub inventory_init: bool,
}

/// Contains the patch queue and patch cache.
#[derive(Debug)]
pub struct EnvironmentCache<Land> {
    /// contains unprocessed patches that are yet to have their dependencies met.
    /// The dependencies are the required patches that live on their three corners.
    /// if the north, east and diagonal patches have not loaded in yet, they will remain in
    /// the patch queue until they come in.
    pub patch_queue: HashMap<U16Vec2, Land>,
    /// All of the patches that been received this session.
    pub patch_cache: HashMap<U16Vec2, Land>,
}

/// Ensure a directory exists
fn create_sub_dir(base: &Path, name: &str) -> Result<PathBuf, SessionError> {
    let dir = base.join(name);
    create_dir_all(&dir).map_err(|e| SessionError::DirCreation {
        dir: dir.clone(),
        error: e,
    })?;
    Ok(dir)
}

/// Initialize the viewer's cache in the share dir on disk
pub fn initialize_share_dir() -> Result<PathBuf, SessionError> {
    let data_dir = dirs::data_dir().ok_or(SessionError::NotFound {})?;

    let share_dir = data_dir.join("benthic");

    create_dir_all(&share_dir).map_err(|e| SessionError::DirCreation {
        dir: share_dir.clone(),
        error: e,
    })?;
    Ok(share_dir)
}

/// Create a subdirectory in the benthic share dir
pub fn create_sub_share_dir(name: &str) -> Result<PathBuf, SessionError> {
    let share_dir = initialize_share_dir()?;
    create_sub_dir(&share_dir, name)
}

/// Create a subdirectory for user agents
pub fn create_sub_agent_dir(name: &str) -> Result<PathBuf, SessionError> {
    let agent_dir = create_sub_share_dir("agent")?;
    create_sub_dir(&agent_dir, name)
}

/// Create a subdirectory for global objects
pub fn create_sub_object_dir(name: &str) -> Result<PathBuf, SessionError> {
    let land_dir = create_sub_share_dir("object")?;
    create_sub_dir(&land_dir, name)
}

/// Create a subdirectory for user agents
pub fn create_sub_land_dir() -> Result<PathBuf, SessionError> {
    create_sub_share_dir("land")
}
/// Create the global animations directory.
pub fn create_animation_dir() -> Result<PathBuf, SessionError> {
    let share_dir = initialize_share_dir()?;
    create_sub_dir(&share_dir, "animations")
}

/// Create the directory containing shared filtered animations.
pub fn create_filtered_animations_dir() -> Result<PathBuf, SessionError> {
    let animations_dir = create_animation_dir()?;
    create_sub_dir(&animations_dir, "filtered_animations")
}

/// Create the directory for a specific filtered animation.
pub fn create_filtered_animation_dir(animation_id: &Uuid) -> Result<PathBuf, SessionError> {
    let filtered_dir = create_filtered_animations_dir()?;
    create_sub_dir(&filtered_dir, &animation_id.to_string())
}

/// Create the directory containing agent-specific animations.
pub fn create_animation_agents_dir() -> Result<PathBuf, SessionError> {
    let animations_dir = create_animation_dir()?;
    create_sub_dir(&animations_dir, "agents")
}

/// Create the directory for a specific agent's animations.
pub fn create_agent_animation_dir(agent_id: &Uuid) -> Result<PathBuf, SessionError> {
    let agents_dir = create_animation_agents_dir()?;
    create_sub_dir(&agents_dir, &agent_id.to_string())
}

pub enum CacheDir {
    Agent(Uuid),
    Object(Uuid),
    Land,
}

pub fn write_json<T: Serialize>(
    data: &T,
    filename: &str,
    cache_dir: CacheDir,
) -> Result<PathBuf, SessionError> {
    let dir = match cache_dir {
        CacheDir::Agent(id) => create_sub_agent_dir(&id.to_string())?,
        CacheDir::Object(id) => create_sub_object_dir(&id.to_string())?,
        CacheDir::Land => create_sub_land_dir()?,
    };

    let path = dir.join(format!("{filename}.json"));

    let json =
        serde_json::to_string(data).map_err(|error| SessionError::JsonWriteError { error })?;

    let mut file = File::create(&path)?;
    file.write_all(json.as_bytes())?;

    Ok(path)
}

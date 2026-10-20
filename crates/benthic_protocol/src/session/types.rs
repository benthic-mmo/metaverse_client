use std::{collections::HashMap, path::PathBuf, sync::Arc};

use glam::{U16Vec2, Vec2};
use uuid::Uuid;

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
pub struct Session<Capability, Avatar, Land, UdpSocket, Inventory, Cache> {
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
    pub inventory: Inventory,
    pub cache: Cache,

    pub share_dir_root: PathBuf,
}

#[derive(Clone, Debug)]
pub enum DownloadState {
    Downloading,
    Complete(PathBuf),
    Failed,
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

    pub current_outfit_root: Uuid,
    pub current_outfit_init: bool,
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

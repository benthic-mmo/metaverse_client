use std::{
    collections::HashMap,
    fs,
    net::{IpAddr, Ipv4Addr},
    path::PathBuf,
};

use benthic_protocol::session::{
    EnvironmentCache, InventoryData, RegionData, Session, set_cache_enabled,
};
use glam::Vec2;
use metaverse_environment::land::Land;
use uuid::{Uuid, uuid};
#[derive(Debug, Default)]
pub struct MockInventory;

#[derive(Debug, Default)]
pub struct MockCache;

pub const AGENT_ID: Uuid = uuid!("96ce3a85-273b-4a1a-b300-78bce703c325");
pub fn mock_session() -> Session<(), (), Land, (), MockInventory, MockCache> {
    set_cache_enabled(false);

    let generated_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/generated");
    fs::create_dir_all(&generated_dir).unwrap();

    Session {
        inventory: MockInventory,
        cache: MockCache,
        agent_id: AGENT_ID,
        session_id: Uuid::nil(),
        address: String::new(),
        seed_capability_url: String::new(),
        sequence_number: 0,
        local_ip: IpAddr::V4(Ipv4Addr::UNSPECIFIED),
        capability_urls: HashMap::new(),

        region_data: RegionData {
            region_coordinates: Vec2::ZERO,
            region_id: String::new(),
            ..Default::default()
        },

        environment_cache: EnvironmentCache {
            patch_queue: HashMap::new(),
            patch_cache: HashMap::new(),
        },

        inventory_data: InventoryData {
            inventory_root: Uuid::nil(),
            inventory_lib_owner: Uuid::nil(),
            inventory_init: true,
        },

        socket: None,
        avatars: HashMap::new(),
        share_dir_root: generated_dir,
    }
}

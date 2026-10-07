use ::bevy::ecs::resource::Resource;
use benthic_default_asset_converter::generated::DEFAULT_SKELETON;
use benthic_protocol::{
    render_data::AvatarObject,
    session::{EnvironmentCache, InventoryData, RegionData, Session, set_cache_enabled},
};
use glam::{Vec2, Vec3};
use metaverse_avatar::{
    avatar::{Avatar, OutfitObject},
    avatar_object_handler::{AvatarState, add_object_to_avatar, finalize_avatar, init_avatar},
};
use metaverse_messages::http::{mesh::Mesh, scene::SceneGroup};
use metaverse_objects::object_handler::create_render_object;
use metaverse_store::initialize_sqlite::{Cache, Inventory};
use std::{
    collections::{BTreeSet, HashMap},
    fs,
    net::{IpAddr, Ipv4Addr},
    path::{Path, PathBuf},
    sync::Arc,
};
use uuid::{Uuid, uuid};

pub mod bevy;

#[derive(Debug, Resource)]
pub struct CreationArtifacts {
    pub avatar_list: HashMap<Uuid, Avatar>,
    pub avatar_object: AvatarObject,
    pub gltf_object: PathBuf,
}

impl CreationArtifacts {
    fn new() -> CreationArtifacts {
        CreationArtifacts {
            avatar_list: HashMap::new(),
            avatar_object: AvatarObject {
                objects: Vec::new(),
                global_skeleton: DEFAULT_SKELETON.clone(),
                used_joints: BTreeSet::new(),
            },
            gltf_object: PathBuf::new(),
        }
    }
}

pub const AGENT_ID: Uuid = uuid!("96ce3a85-273b-4a1a-b300-78bce703c325");

async fn mock_session(generated_dir: PathBuf) -> Session<(), Avatar, (), (), Inventory, Cache> {
    let inventory_db_connection = sqlx::SqlitePool::connect(":memory:").await.unwrap();

    let inventory = Inventory::new(inventory_db_connection.clone());
    let cache = Cache::new(inventory_db_connection);

    Session {
        inventory,
        cache,
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
            current_outfit_root: Uuid::nil(),
            current_outfit_init: true,
        },

        socket: None,
        share_dir_root: generated_dir,

        #[cfg(feature = "avatar")]
        avatars: HashMap::new(),

        downloads: Arc::new(tokio::sync::Mutex::new(HashMap::new())),
    }
}

fn mock_from_xml(
    session: &mut Session<(), Avatar, (), (), Inventory, Cache>,
    scene_group_path: &Path,
) -> Result<AvatarState, Box<dyn std::error::Error>> {
    // SceneGroup is raw XML received from the server.
    let buffer = fs::read(scene_group_path)?;
    let scene_group = SceneGroup::from_xml(&buffer)?;

    let mesh_dir =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/data/raw_server_data/meshes");

    let texture_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/data/textures");

    let mut render_objects = Vec::new();

    for part in &scene_group.parts {
        println!("generating: {:?}", part.metadata.name);

        // The sculpt texture UUID identifies the mesh asset.
        let mesh_path = mesh_dir.join(format!("{}.bin", part.sculpt.texture));

        let mesh_bytes = fs::read(&mesh_path)?;
        let mesh = Mesh::from_bytes(&mesh_bytes)?;

        let texture_path = texture_dir.join(format!("{}.png", part.shape.texture.texture_id));

        let render_object = create_render_object(
            mesh,
            part.metadata.name.clone(),
            &texture_path,
            part.sculpt.texture,
        )?;

        render_objects.push(render_object);
    }

    let json_name = format!(
        "{:?}_{}.json",
        scene_group.parts[0].sculpt.texture, scene_group.parts[0].metadata.name
    );

    let generated_agent_dir = session.share_dir_root.join(AGENT_ID.to_string());

    fs::create_dir_all(&generated_agent_dir)?;

    let json_path = generated_agent_dir.join(json_name);

    let json = serde_json::to_vec(&render_objects)?;
    fs::write(&json_path, json)?;

    let state = add_object_to_avatar(session, AGENT_ID, OutfitObject::MeshObject(json_path))?;

    Ok(state)
}

async fn mock_finalize_avatar(
    session: &mut Session<(), Avatar, (), (), Inventory, Cache>,
) -> Result<PathBuf, Box<dyn std::error::Error>> {
    let avatar = session.avatars.get_mut(&AGENT_ID).unwrap();

    let (glb_path, skeleton) = finalize_avatar(
        AGENT_ID,
        avatar.skeleton.clone(),
        avatar.used_joints.clone(),
        avatar.items.clone(),
        session.share_dir_root.clone(),
    )
    .await?;

    avatar.path = Some(glb_path.clone());
    avatar.skeleton = skeleton;

    Ok(glb_path)
}

pub async fn mock_avatar_load() -> CreationArtifacts {
    set_cache_enabled(false);

    let mut artifacts = CreationArtifacts::new();

    let generated_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/generated");

    fs::create_dir_all(&generated_dir).unwrap();

    let mut session = mock_session(generated_dir.clone()).await;

    let mut avatar = Avatar::new(AGENT_ID, Vec3::ZERO);
    avatar.outfit_size = 4;

    init_avatar(&mut session, &avatar).unwrap();

    let scene_groups_path =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/data/raw_server_data/scenegroups");

    for entry in fs::read_dir(scene_groups_path).unwrap() {
        let path = entry.unwrap().path();

        if path.extension().is_none_or(|ext| ext != "xml") {
            continue;
        }

        let state = mock_from_xml(&mut session, &path).unwrap();

        if matches!(state, AvatarState::FullyLoaded) {
            let glb_path = mock_finalize_avatar(&mut session).await.unwrap();

            artifacts.gltf_object = glb_path;
        }
    }

    let avatar = session.avatars.get(&AGENT_ID).unwrap();

    artifacts.avatar_object = AvatarObject {
        objects: avatar
            .items
            .iter()
            .filter_map(|item| match item {
                OutfitObject::MeshObject(path) => Some(path.clone()),
                _ => None,
            })
            .collect(),
        global_skeleton: avatar.skeleton.clone(),
        used_joints: avatar.used_joints.clone(),
    };

    artifacts
}

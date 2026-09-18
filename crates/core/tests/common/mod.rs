use ::bevy::ecs::resource::Resource;
use benthic_protocol::{
    render_data::{AvatarObject, RenderObject},
    session::{EnvironmentCache, InventoryData, RegionData, Session, set_cache_enabled},
};
use default_asset_converter::generated::DEFAULT_SKELETON;
use glam::{Vec2, Vec3};
use metaverse_avatar::{
    avatar::{Avatar, OutfitObject},
    avatar_object_handler::{AvatarState, add_object_to_avatar, finalize_avatar, init_avatar},
};
use metaverse_messages::http::{mesh::Mesh, scene::SceneGroup};
use metaverse_objects::object_handler::create_render_object;
use std::{
    collections::{BTreeSet, HashMap},
    fs::{self, File},
    io::Write,
    net::{IpAddr, Ipv4Addr},
    path::PathBuf,
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
pub async fn mock_avatar_load() -> CreationArtifacts {
    set_cache_enabled(false);
    let mut artifacts = CreationArtifacts::new();
    let inventory_db_connection = sqlx::SqlitePool::connect(":memory:").await.unwrap();

    let mut session: Session<(), Avatar, (), ()> = Session {
        inventory_db_connection,
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

        #[cfg(feature = "avatar")]
        avatars: HashMap::new(),
    };
    session.agent_id = AGENT_ID;
    session.inventory_data.inventory_init = true;

    let mut avatar = Avatar::new(AGENT_ID, Vec3::ZERO);
    avatar.outfit_size = 4;

    init_avatar(&mut session, &avatar).unwrap();

    let generated_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/generated");
    fs::create_dir_all(&generated_dir).unwrap();

    let generated_agent_dir = generated_dir.join(AGENT_ID.to_string());
    fs::create_dir_all(&generated_agent_dir).unwrap();

    let scene_groups_path =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/data/scene_groups");

    for entry in fs::read_dir(&scene_groups_path).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();

        if path.extension().is_some_and(|ext| ext != "json") {
            continue;
        }

        let file = fs::File::open(&path).unwrap();

        let scene_group: SceneGroup = serde_json::from_reader(file).unwrap();

        let mut render_objects: Vec<RenderObject> = Vec::new();

        for part in &scene_group.parts {
            println!("generating: {:?}", part.metadata.name);

            let mesh_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join(format!("tests/data/meshes/{}.json", part.sculpt.texture));

            let file = fs::File::open(&mesh_path).unwrap();

            let texture_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(format!(
                "tests/data/textures/{}.png",
                part.shape.texture.texture_id
            ));

            let mesh: Mesh = serde_json::from_reader(file).unwrap();

            let render_object = create_render_object(
                mesh,
                part.metadata.name.clone(),
                &texture_path,
                part.sculpt.texture,
            )
            .unwrap();

            render_objects.push(render_object);
        }

        let json_name = format!(
            "{:?}_{}.json",
            scene_group.parts[0].sculpt.texture, scene_group.parts[0].metadata.name
        );

        let json_path = generated_agent_dir.join(json_name);

        let json = serde_json::to_string(&render_objects).unwrap();

        let mut file = File::create(&json_path).unwrap();
        file.write_all(json.as_bytes()).unwrap();

        let state =
            add_object_to_avatar(&mut session, AGENT_ID, OutfitObject::MeshObject(json_path))
                .unwrap();

        if matches!(state, AvatarState::FullyLoaded) {
            let avatar = session.avatars.get_mut(&AGENT_ID).unwrap();

            let (glb_path, skeleton) = finalize_avatar(
                AGENT_ID,
                avatar.skeleton.clone(),
                avatar.used_joints.clone(),
                avatar.items.clone(),
            )
            .await
            .unwrap();
            avatar.path = Some(glb_path.clone());
            avatar.skeleton = skeleton;

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

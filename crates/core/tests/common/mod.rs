use ::bevy::ecs::resource::Resource;
use benthic_protocol::render_data::{AvatarObject, RenderObject};
use default_asset_converter::generated::DEFAULT_SKELETON;
use glam::Vec3;
use metaverse_avatar::{
    avatar::{Avatar, OutfitObject},
    skeleton::update_global_avatar_skeleton,
};
use metaverse_mesh::mesh::generate::generate_skinned_mesh;
use metaverse_messages::http::{mesh::Mesh, scene::SceneGroup};
use metaverse_objects::object_handler::create_render_object;
use std::{
    collections::{BTreeSet, HashMap},
    fs::{self, File},
    io::Write,
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

pub fn mock_avatar_load() -> CreationArtifacts {
    let mut artifacts = CreationArtifacts::new();
    artifacts
        .avatar_list
        .insert(AGENT_ID, Avatar::new(AGENT_ID, Vec3::ZERO));
    artifacts
        .avatar_list
        .get_mut(&AGENT_ID)
        .unwrap()
        .outfit_size = 4;
    let generated_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/generated");
    if let Err(e) = fs::create_dir_all(&generated_dir) {
        panic!(
            "Failed to create generated directory {:?}: {:?}",
            generated_dir, e
        );
    }
    let generated_agent_dir =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(format!("tests/generated/{}", AGENT_ID));
    if let Err(e) = fs::create_dir_all(&generated_agent_dir) {
        panic!(
            "Failed to create generated directory {:?}: {:?}",
            generated_agent_dir, e
        );
    }

    let scene_groups_path =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/data/scene_groups");

    for entry in fs::read_dir(&scene_groups_path)
        .unwrap_or_else(|e| panic!("Failed to read {:?}: {}", scene_groups_path, e))
    {
        let entry = entry.unwrap_or_else(|e| panic!("Failed to read directory entry: {}", e));
        let path = entry.path();

        if path.extension().is_some_and(|ext| ext != "json") {
            continue;
        }
        let file =
            fs::File::open(&path).unwrap_or_else(|e| panic!("Failed to open {:?}: {}", path, e));

        let scene_group: SceneGroup = serde_json::from_reader(file)
            .unwrap_or_else(|e| panic!("Failed to deserialize {:?}: {}", path, e));

        let mut render_objects: Vec<RenderObject> = Vec::new();

        for part in &scene_group.parts {
            println!("generating: {:?}", part.metadata.name);

            let mut mesh_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
            mesh_path.push(format!("tests/data/meshes/{}.json", part.sculpt.texture));
            let file = fs::File::open(&mesh_path)
                .unwrap_or_else(|e| panic!("Failed to open {:?}: {}", mesh_path, e));

            let mut texture_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
            texture_path.push(format!(
                "tests/data/textures/{}.png",
                part.shape.texture.texture_id
            ));

            let mesh: Mesh = serde_json::from_reader(file)
                .unwrap_or_else(|e| panic!("Failed to deserialize {:?}: {}", mesh_path, e));
            let render_object = create_render_object(
                mesh,
                part.metadata.name.clone(),
                &texture_path,
                part.sculpt.texture,
            )
            .unwrap_or_else(|e| panic!("Render Object creation failed: {:?}", e));
            render_objects.push(render_object);
        }

        let json_name = format!(
            "{:?}_{}.json",
            scene_group.parts[0].sculpt.texture, scene_group.parts[0].metadata.name
        );
        let json_path = generated_agent_dir.join(json_name);
        match serde_json::to_string(&render_objects) {
            Ok(json) => {
                let mut file = File::create(&json_path).unwrap();
                file.write_all(json.as_bytes()).unwrap();
            }
            Err(e) => {
                panic!("Failed to write json :{}", e)
            }
        }
        add_object_to_avatar(
            &mut artifacts,
            AGENT_ID,
            OutfitObject::MeshObject(json_path),
        );
    }
    artifacts
}

fn add_object_to_avatar(artifacts: &mut CreationArtifacts, agent_id: Uuid, object: OutfitObject) {
    let generated_agent_dir =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(format!("tests/generated/{}", AGENT_ID));
    if let Some(avatar) = artifacts.avatar_list.get_mut(&agent_id) {
        match &object {
            OutfitObject::MeshObject(path) => {
                let file = fs::File::open(path)
                    .unwrap_or_else(|e| panic!("Failed to open {:?}: {}", path, e));
                let parts: Vec<RenderObject> = serde_json::from_reader(file)
                    .unwrap_or_else(|e| panic!("Failed to read serde {:?}: {}", path, e));

                if let Some(skin) = &parts[0].skin {
                    update_global_avatar_skeleton(avatar, &skin.skeleton);
                }
            }
            _ => {
                panic!("No tests for non-mesh objects yet")
            }
        }
        avatar.items.push(object);
        if avatar.items.len() == avatar.outfit_size {
            avatar.fully_loaded = true;
            let json_paths: Vec<PathBuf> = avatar
                .items
                .clone()
                .into_iter()
                .filter_map(|item| {
                    if let OutfitObject::MeshObject(path) = item {
                        Some(path)
                    } else {
                        None
                    }
                })
                .collect();

            let avatar_object = AvatarObject {
                objects: json_paths,
                global_skeleton: avatar.skeleton.clone(),
                used_joints: avatar.used_joints.clone(),
            };

            artifacts.avatar_object = avatar_object.clone();
            let json_path =
                generated_agent_dir.join(format!("{}_avatar_object.json", avatar.agent_id));
            match serde_json::to_string(&avatar_object) {
                Ok(json) => {
                    let mut file = File::create(&json_path).unwrap();
                    file.write_all(json.as_bytes()).unwrap();
                }
                Err(e) => {
                    panic!("Failed to write json :{}", e)
                }
            }
            let glb_path = generated_agent_dir.join(format!("{:?}_high.glb", avatar.agent_id));
            artifacts.gltf_object = glb_path.clone();
            println!("{:?}", glb_path);
            println!("Generating skinned mesh");
            generate_skinned_mesh(json_path.clone(), glb_path.clone()).unwrap()
        }
    }
}

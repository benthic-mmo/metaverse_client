// skeleton.rs dependencies
use glam::vec3;
use metaverse_agent::{
    avatar::{Avatar, OutfitObject, RiggedObject},
    generate_gltf::generate_baked_avatar,
    skeleton::{create_skeleton, update_global_avatar_skeleton},
};
use metaverse_messages::capabilities::scene::SceneGroup;
use std::{
    collections::HashMap,
    fs::{self, File},
    io::Write,
    path::PathBuf,
    sync::{Arc, Mutex},
};
use uuid::uuid;

#[test]
pub fn combine_gltf() {
    let agent_list = Arc::new(Mutex::new(HashMap::new()));
    let mut elements = vec![];
    let mut rigged_objects = vec![];

    let agent_uuid = uuid!("45b5a67d-8753-46e5-9408-8b47e8a6e48f");
    // create a new agent with a default UUID. This allows the skeleton to be updated properly as
    // mocked clothing items are parsed and loaded in.
    {
        let newagent = Avatar::new(agent_uuid, vec3(0.0, 0.0, 0.0), 0);
        agent_list.lock().unwrap().insert(agent_uuid, newagent);
    }

    // get the body and T-shirt objects
    let mut overalls_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    overalls_path.push("tests/example_sceneobjects/overalls.json");
    let mut shirt_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    shirt_path.push("tests/example_sceneobjects/t-shirt.json");
    let mut body_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    body_path.push("tests/example_sceneobjects/body.json");
    let mut curves_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    curves_path.push("tests/example_sceneobjects/curves.json");

    // get the skeleton path and the path where the generated gltf goes
    let mut skeleton_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    skeleton_path.push("tests/example_sceneobjects/skeleton.gltf");
    let mut agent_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    agent_path.push("tests/generated_gltf");

    let mut combined_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    combined_path.push("tests/generated_gltf/Combined.glb");

    let mut skeleton_json_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    skeleton_json_path.push("tests/generated_gltf/skeleton.json");

    // get strings
    let body_str = fs::read_to_string(&body_path).expect("Failed to read body.json");
    let shirt_str = fs::read_to_string(&shirt_path).expect("Failed to read t-shirt.json");
    let overalls_str = fs::read_to_string(&overalls_path).expect("Failed to read t-shirt.json");
    let curves_str = fs::read_to_string(&curves_path).expect("Failed to read curves.json");

    // deserialize
    let body: SceneGroup =
        serde_json::from_str(&body_str).expect("Failed to deserialize body.json");
    let shirt: SceneGroup =
        serde_json::from_str(&shirt_str).expect("Failed to deserialize shirt.json");
    let overalls: SceneGroup =
        serde_json::from_str(&overalls_str).expect("Failed to deserialize shirt.json");
    let curves: SceneGroup =
        serde_json::from_str(&curves_str).expect("Failed to deserialize shirt.json");

    elements.push(shirt);
    elements.push(curves);
    elements.push(overalls);
    elements.push(body);

    for element in elements {
        // generate the skeleton for each object. This updates the global agent skeleton object with the correct
        // joint transforms, and stores the object specific skeleton and transforms.
        let local_skeleton = create_skeleton(element.parts[0].clone()).unwrap();

        if let Some(agent) = agent_list.lock().unwrap().get_mut(&agent_uuid) {
            update_global_avatar_skeleton(agent, &local_skeleton);
        }

        rigged_objects.push(OutfitObject::RiggedObject(RiggedObject {
            scene_group: element,
            skeleton: local_skeleton,
        }))
    }

    // display the global skeleton transform for the neck
    {
        let final_skeleton = agent_list
            .lock()
            .unwrap()
            .get(&agent_uuid)
            .unwrap()
            .skeleton
            .clone();
        let json = serde_json::to_string_pretty(&final_skeleton).expect("Failed to serialize");

        let mut file = File::create(skeleton_json_path).unwrap();
        file.write_all(json.as_bytes());
        generate_baked_avatar(rigged_objects, final_skeleton, combined_path);
    }
}

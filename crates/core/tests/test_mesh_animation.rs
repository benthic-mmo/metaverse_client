use benthic_protocol::default_animations::AnimationClip;
use benthic_protocol::skeleton::{JointName, Skeleton};
use bevy::app::Update;
use bevy::asset::{AssetMode, AssetPlugin, UnapprovedPathMode};
use bevy::ecs::prelude::*;
use bevy::winit::WinitPlugin;
use bevy::{
    DefaultPlugins,
    animation::AnimationPlayer,
    app::{App, PluginGroup, Startup},
    asset::{AssetServer, Assets, Handle},
    ecs::system::{Commands, Query, Res, ResMut},
    gltf::GltfAssetLabel,
    prelude::{AnimationGraph, AnimationGraphHandle, AnimationNodeIndex, Resource},
};
use bevy_panorbit_camera::PanOrbitCameraPlugin;
use default_asset_converter::generated_asset_path;
use lazy_static::lazy_static;
use metaverse_avatar::animation::{apply_joint_scale, filter_animation};
use metaverse_mesh::animation::generate::generate_gltf_animation;
use metaverse_mesh::animation::gltf::export_animation;
use std::collections::BTreeSet;
use std::path::PathBuf;

use crate::common::bevy::{
    DebugVisibility, draw_skeleton_debug, handle_ui_buttons, setup, update_mesh_visibility,
};
use crate::common::{AGENT_ID, mock_avatar_load};
mod common;

lazy_static! {
    static ref PUFFBALL_JOINT_FILTER: BTreeSet<JointName> = BTreeSet::from([
        JointName::Pelvis,
        JointName::Torso,
        JointName::Tail1,
        JointName::HipLeft,
        JointName::HipRight,
        JointName::Chest,
        JointName::Neck,
        JointName::CollarLeft,
        JointName::CollarRight,
        JointName::Head,
        JointName::Skull,
        JointName::FaceRoot,
        JointName::FaceJaw,
        JointName::FaceJawShaper,
        JointName::FaceEar1Left,
        JointName::FaceEar1Right,
        JointName::FaceEar2Left,
        JointName::FaceEar2Right,
        JointName::ShoulderLeft,
        JointName::ElbowLeft,
        JointName::WristLeft,
        JointName::HandIndex1Left,
        JointName::HandMiddle1Left,
        JointName::HandRing1Left,
        JointName::HandPinky1Left,
        JointName::HandThumb1Left,
        JointName::HandIndex2Left,
        JointName::HandMiddle2Left,
        JointName::HandRing2Left,
        JointName::HandPinky2Left,
        JointName::HandThumb2Left,
        JointName::HandIndex3Left,
        JointName::HandMiddle3Left,
        JointName::HandRing3Left,
        JointName::HandPinky3Left,
        JointName::HandThumb3Left,
        JointName::ShoulderRight,
        JointName::ElbowRight,
        JointName::WristRight,
        JointName::HandIndex1Right,
        JointName::HandMiddle1Right,
        JointName::HandRing1Right,
        JointName::HandPinky1Right,
        JointName::HandThumb1Right,
        JointName::HandIndex2Right,
        JointName::HandMiddle2Right,
        JointName::HandRing2Right,
        JointName::HandPinky2Right,
        JointName::HandThumb2Right,
        JointName::HandIndex3Right,
        JointName::HandMiddle3Right,
        JointName::HandRing3Right,
        JointName::HandPinky3Right,
        JointName::HandThumb3Right,
        JointName::Tail1,
        JointName::Tail2,
        JointName::Tail3,
        JointName::Tail4,
        JointName::Tail5,
        JointName::Tail6,
        JointName::KneeLeft,
        JointName::AnkleLeft,
        JointName::FootLeft,
        JointName::KneeRight,
        JointName::AnkleRight,
        JointName::FootRight,
    ]);
}

fn generated_animation_path(name: &str) -> PathBuf {
    let path = PathBuf::from("tests")
        .join("generated")
        .join(AGENT_ID.to_string())
        .join("animation");

    std::fs::create_dir_all(&path).unwrap();

    path.join(name)
}

fn load_animation(name: &str, target_skeleton: &Skeleton) -> AnimationClip {
    let path = generated_asset_path();
    let filename = path.join("Animations").join(format!("{name}.json"));

    println!("loading animation: {:?}", filename);

    let filtered_animation_out_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("generated")
        .join("animation")
        .join(format!("{name}_filtered.json"));

    filter_animation(
        &filename,
        &filtered_animation_out_path.clone(),
        PUFFBALL_JOINT_FILTER.clone(),
    )
    .unwrap();

    let agent_animation_out_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("generated")
        .join("animation")
        .join(format!("{name}_agent_scaled.json"));

    apply_joint_scale(
        &filtered_animation_out_path,
        agent_animation_out_path.clone(),
        target_skeleton,
    )
    .unwrap();

    serde_json::from_reader(
        std::fs::File::open(&agent_animation_out_path)
            .expect("failed to open scaled animation json"),
    )
    .expect("failed to deserialize scaled animation json")
}

#[test]
fn run_stand() {
    display_animation("Stand");
}

#[test]
fn run_dance() {
    display_animation("Dance");
}

#[test]
fn build_animation() {
    let artifacts = mock_avatar_load();

    let path = generated_asset_path();
    let filename = path.join("Animations").join("Stand.json");

    // Filtered intermediate JSON.
    let filtered_animation_out_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("generated")
        .join("animation")
        .join("Stand_filtered.json");

    filter_animation(
        &filename,
        &filtered_animation_out_path.clone(),
        PUFFBALL_JOINT_FILTER.clone(),
    )
    .unwrap();

    // Avatar-specific scaled JSON.
    let agent_animation_out_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("generated")
        .join("animation")
        .join("Stand_agent_scaled.json");

    apply_joint_scale(
        &filtered_animation_out_path,
        agent_animation_out_path.clone(),
        &artifacts.avatar_object.global_skeleton,
    )
    .unwrap();

    // Final GLB.
    let out_path = generated_animation_path("Stand.glb");

    generate_gltf_animation(&agent_animation_out_path, &out_path).unwrap();
}

#[derive(Debug, Resource)]
struct AnimationName(String);

fn display_animation(animation: &str) {
    let artifacts = mock_avatar_load();

    let out_path = generated_animation_path(&format!("{}.glb", animation));

    let animations = load_animation(animation, &artifacts.avatar_object.global_skeleton);

    export_animation(&animations, &out_path.clone()).unwrap();

    let tests_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join(format!("tests/generated/{}", common::AGENT_ID));

    let mut app = App::new();

    app.add_plugins(
        DefaultPlugins
            .set(WinitPlugin {
                run_on_any_thread: true,
            })
            .set(AssetPlugin {
                file_path: tests_dir.into_string().unwrap(),
                mode: AssetMode::Unprocessed,
                unapproved_path_mode: UnapprovedPathMode::Allow,
                ..Default::default()
            }),
    );

    app.add_plugins(PanOrbitCameraPlugin);

    app.insert_resource(DebugVisibility {
        show_mesh: true,
        show_skeleton: true,
        show_joint_axes: false,
    });

    app.insert_resource(artifacts);
    app.insert_resource(AnimationName(animation.to_string()));

    app.add_systems(Startup, setup);
    app.add_systems(Update, handle_ui_buttons);
    app.add_systems(Update, update_mesh_visibility);
    app.add_systems(Update, draw_skeleton_debug);
    app.add_systems(Startup, setup_animation_graph);

    app.add_observer(animation_player_added);

    app.run();
}

fn setup_animation_graph(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut graphs: ResMut<Assets<AnimationGraph>>,
    animation_name: Res<AnimationName>,
) {
    let mut graph = AnimationGraph::new();

    let animation_path = format!("animation/{}.glb", animation_name.0);

    let animations = vec![graph.add_clip(
        asset_server.load(GltfAssetLabel::Animation(0).from_asset(animation_path)),
        1.0,
        graph.root,
    )];

    let graph_handle = graphs.add(graph);

    commands.insert_resource(AnimationGraphCache {
        animations,
        graph: graph_handle,
    });
}

#[derive(Debug, Resource)]
struct AnimationGraphCache {
    animations: Vec<AnimationNodeIndex>,
    graph: Handle<AnimationGraph>,
}

fn animation_player_added(
    trigger: On<Add, AnimationPlayer>,
    mut commands: Commands,
    graph_cache: Res<AnimationGraphCache>,
    mut players: Query<&mut AnimationPlayer>,
) {
    if let Ok(mut player) = players.get_mut(trigger.entity) {
        player.play(graph_cache.animations[0]).repeat();

        commands
            .entity(trigger.entity)
            .insert(AnimationGraphHandle(graph_cache.graph.clone()));
    }
}

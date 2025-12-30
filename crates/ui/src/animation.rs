use crate::render::AgentIDMap;
use bevy::{
    animation::{graph::AnimationGraph, AnimationPlayer},
    asset::{AssetServer, Assets, Handle, LoadState},
    ecs::{
        resource::Resource,
        system::{Query, ResMut},
    },
    platform::collections::HashMap,
};
use bevy_gltf::Gltf;
use std::path::PathBuf;
use uuid::Uuid;

#[derive(Resource)]
pub struct AnimationQueue {
    pub pending: HashMap<Uuid, AnimationPath>,
}

pub struct AnimationPath {
    pub path_on_disk: PathBuf,
    pub gltf_handle: Handle<Gltf>,
}

pub fn update_animations(
    mut animation_queue: ResMut<AnimationQueue>,
    agent_id_map: ResMut<AgentIDMap>,
    asset_server: ResMut<AssetServer>,
    mut players: Query<&mut AnimationPlayer>,
    mut graphs: ResMut<Assets<AnimationGraph>>,
    gltf_assets: ResMut<Assets<Gltf>>,
) {
    let mut done = vec![];

    for (agent_id, paths) in animation_queue.pending.iter() {
        if let Some(agent) = agent_id_map.entities.get(agent_id) {
            if asset_server.load_state(&paths.gltf_handle).is_loaded() {
                println!("gltf handle {:?}", gltf_assets.get(&paths.gltf_handle));
                if let Some(gltf) = gltf_assets.get(&paths.gltf_handle) {
                    if let Some(anim_handle) = gltf.animations.first() {
                        if let Ok(mut player) = players.get_mut(agent.entity) {
                            let (graph, node_index) =
                                AnimationGraph::from_clip(anim_handle.clone());
                            graphs.add(graph);
                            player.start(node_index);
                        }
                    }
                    done.push(*agent_id);
                }
            }
        }
    }

    for id in done {
        animation_queue.pending.remove(&id);
    }
}

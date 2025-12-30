use crate::render::AgentIDMap;
use bevy::{
    animation::{graph::AnimationGraph, AnimationPlayer},
    asset::{AssetServer, Assets, Handle},
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
    pub pending: HashMap<Uuid, PathBuf>,
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

    for (agent_id, queued) in animation_queue.pending.iter() {
        if let Some(agent) = agent_id_map.entities.get(agent_id) {
            let path_str = queued.to_string_lossy().to_string();

            // Load as Gltf asset, not AnimationClip
            let gltf_handle: Handle<Gltf> = asset_server.load(path_str);

            // Check if the Gltf asset is loaded and has animations
            if let Some(gltf) = gltf_assets.get(&gltf_handle) {
                // Get the first animation (or use your preferred selection logic)
                if let Some(anim_handle) = gltf.animations.first() {
                    if let Ok(mut player) = players.get_mut(agent.entity) {
                        let (graph, node_index) = AnimationGraph::from_clip(anim_handle.clone());
                        let graph_handle = graphs.add(graph);
                        player.start(node_index);
                    }
                }
                done.push(*agent_id);
            }
        }
    }

    for id in done {
        animation_queue.pending.remove(&id);
    }
}

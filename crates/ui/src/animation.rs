use crate::render::AgentIDMap;
use bevy::animation::graph::{AnimationNodeIndex, AnimationNodeType};
use bevy::ecs::component::Component;
use bevy::prelude::Res;
use bevy::{
    animation::{
        graph::{AnimationGraph, AnimationGraphHandle},
        AnimationPlayer,
    },
    asset::{Assets, Handle},
    ecs::{
        resource::Resource,
        system::{Commands, Query, ResMut},
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

#[derive(Component)]
pub struct PlayingAnimationIndex(AnimationNodeIndex);

pub fn update_animations(
    mut animation_queue: ResMut<AnimationQueue>,
    mut agent_id_map: ResMut<AgentIDMap>,
    gltf_assets: Res<Assets<Gltf>>,
    mut graphs: ResMut<Assets<AnimationGraph>>,
    mut commands: Commands,
    mut players: Query<&mut AnimationPlayer>,
) {
    let mut done = vec![];

    for (agent_id, animation_data) in animation_queue.pending.iter() {
        if let Some(agent) = agent_id_map.entities.get_mut(agent_id) {
            if let Some(gltf) = gltf_assets.get(&animation_data.gltf_handle) {
                if let Some(anim_handle) = gltf.animations.first() {
                    if let Ok(mut player) = players.get_mut(agent.skeleton) {
                        let (graph, node_index) = AnimationGraph::from_clip(anim_handle.clone());
                        let graph_handle = graphs.add(graph);

                        commands
                            .entity(agent.skeleton)
                            .insert(AnimationGraphHandle(graph_handle))
                            .insert(PlayingAnimationIndex(node_index));

                        println!("node index 1{:?}", node_index);
                        player.start(node_index);

                        println!(
                            "Started animation {:?} on agent {:?}",
                            anim_handle, agent_id
                        );

                        done.push(*agent_id);
                    } else {
                        // If no AnimationPlayer yet, insert and try again  .insert(PlayingAnimationIndex(node_index));next frame
                        commands
                            .entity(agent.skeleton)
                            .insert(AnimationPlayer::default());
                        println!(
                            "Inserted AnimationPlayer on skeleton for agent {:?}",
                            agent_id
                        );
                    }
                } else {
                    println!("GLTF has no animations: {:?}", animation_data.gltf_handle);
                }
            } else {
                println!(
                    "GLTF asset not yet loaded: {:?}",
                    animation_data.gltf_handle
                );
            }
        }
    }

    // Remove processed animations
    for id in done {
        animation_queue.pending.remove(&id);
    }
}

pub fn apply_animation_graphs(
    graphs: Res<Assets<AnimationGraph>>,
    mut players: Query<(
        &AnimationGraphHandle,
        &mut AnimationPlayer,
        &PlayingAnimationIndex,
    )>,
) {
    for (graph_handle, mut player, playing_index) in &mut players {
        if let Some(graph_asset) = graphs.get(&graph_handle.0) {
            let node_index = playing_index.0;

            if let Some(root_node) = graph_asset.graph.node_weight(node_index) {
                let clip_handle = match &root_node.node_type {
                    AnimationNodeType::Clip(handle) => handle.clone(),
                    _ => continue,
                };

                let is_playing = player.is_playing_animation(node_index);
                if !is_playing {
                    println!("Starting animation clip: {:?}", clip_handle);
                    player.play(node_index).repeat();
                }
            }
        }
    }
}

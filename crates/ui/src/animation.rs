use crate::core_plugin::AnimationEvent;
use crate::mesh::AgentID;
use bevy::app::{App, Plugin, Update};
use bevy::asset::AssetServer;
use bevy::ecs::component::Component;
use bevy::ecs::entity::Entity;
use bevy::ecs::message::MessageReader;
use bevy::ecs::observer::On;
use bevy::gltf::Gltf;
use bevy::prelude::Res;
use bevy::{
    animation::{
        AnimationPlayer,
        graph::{AnimationGraph, AnimationGraphHandle},
    },
    asset::{Assets, Handle},
    ecs::{
        resource::Resource,
        system::{Commands, Query, ResMut},
    },
    platform::collections::HashMap,
};
use bevy_world_serialization::WorldInstanceReady;
use log::warn;
use std::path::PathBuf;
use uuid::Uuid;

pub struct BenthicAnimationPlugin;
impl Plugin for BenthicAnimationPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, update_animations)
            .add_systems(Update, handle_animation_update)
            .insert_resource(AnimationQueue {
                pending: HashMap::new(),
            });
    }
}

#[derive(Resource)]
pub struct AnimationQueue {
    pub pending: HashMap<Uuid, AnimationPath>,
}

pub struct AnimationPath {
    pub path_on_disk: PathBuf,
    pub gltf_handle: Handle<Gltf>,
}

#[derive(Component)]
pub struct AgentAnimationPlayer {
    agent_id: Uuid,
}

pub fn scene_instance_ready(
    trigger: On<WorldInstanceReady>,
    mut commands: Commands,
    animation_players: Query<(Entity, &AnimationPlayer)>,
    agent_id_query: Query<(Entity, &AgentID)>,
) {
    if let Some((_, agent_id)) = agent_id_query
        .iter()
        .find(|(entity, _)| *entity == trigger.entity)
    {
        for (player_entity, _) in animation_players.iter() {
            commands.entity(player_entity).insert(AgentAnimationPlayer {
                agent_id: agent_id.id,
            });
            println!(
                "Mapped AnimationPlayer {:?} to agent {:?}",
                player_entity, agent_id.id
            );
        }
    }
}

pub fn handle_animation_update(
    mut events: MessageReader<AnimationEvent>,
    mut animation_queue: ResMut<AnimationQueue>,
    asset_server: Res<AssetServer>,
) {
    for event in events.read() {
        let gltf_handle: Handle<Gltf> = asset_server.load(event.value.animation_path.clone());
        animation_queue.pending.insert(
            event.value.player_id,
            AnimationPath {
                path_on_disk: event.value.animation_path.clone(),
                gltf_handle,
            },
        );
    }
}

pub fn update_animations(
    mut animation_queue: ResMut<AnimationQueue>,
    gltf_assets: Res<Assets<Gltf>>,
    mut graphs: ResMut<Assets<AnimationGraph>>,
    mut commands: Commands,
    mut players: Query<(Entity, &mut AnimationPlayer, &AgentAnimationPlayer)>,
) {
    let mut done = vec![];

    for (agent_id, animation_data) in animation_queue.pending.iter() {
        for (player_entity, mut player, agent_animation) in players.iter_mut() {
            if agent_animation.agent_id == *agent_id {
                let Some(gltf) = gltf_assets.get(&animation_data.gltf_handle) else {
                    println!("GLTF not loaded yet: {:?}", animation_data.gltf_handle);
                    continue;
                };

                let Some(anim_handle) = gltf.animations.first() else {
                    println!("No animations in GLTF: {:?}", animation_data.gltf_handle);
                    continue;
                };

                let (graph, node_index) = AnimationGraph::from_clip(anim_handle.clone());
                let graph_handle = graphs.add(graph);

                commands
                    .entity(player_entity)
                    .insert(AnimationGraphHandle(graph_handle));
                player.play(node_index).repeat();

                println!(
                    "Playing animation for {:?} at node {:?}",
                    agent_id, node_index
                );
                done.push(*agent_id);
            } else {
                warn!("Failed to find player entity for animation");
            }
        }
    }
    for id in done {
        animation_queue.pending.remove(&id);
    }
}

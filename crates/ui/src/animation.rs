use crate::avatar::AgentID;
use crate::core_plugin::AnimationEvent;
use bevy::app::{App, Plugin, Update};
use bevy::asset::AssetServer;
use bevy::ecs::component::Component;
use bevy::ecs::entity::Entity;
use bevy::ecs::hierarchy::{ChildOf, Children};
use bevy::ecs::message::MessageReader;
use bevy::ecs::observer::On;
use bevy::ecs::query::With;
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
use log::{info, warn};
use std::path::PathBuf;
use uuid::Uuid;

pub struct BenthicAnimationPlugin;
impl Plugin for BenthicAnimationPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(scene_instance_ready)
            .add_systems(Update, update_animations)
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
    agent_ids: Query<&AgentID>,
    parents: Query<&ChildOf>,
    children: Query<&Children>,
    animation_players: Query<(), With<AnimationPlayer>>,
) {
    // Find the avatar ID on this entity or one of its ancestors.
    let mut current = trigger.entity;

    let agent_id = loop {
        if let Ok(agent_id) = agent_ids.get(current) {
            break Some(agent_id.id);
        }

        let Ok(parent) = parents.get(current) else {
            break None;
        };

        current = parent.parent();
    };

    let Some(agent_id) = agent_id else {
        warn!(
            "WorldInstanceReady: no AgentID found for {:?}",
            trigger.entity
        );
        return;
    };

    // Walk only this instance's descendants.
    let mut pending = vec![trigger.entity];

    while let Some(entity) = pending.pop() {
        if animation_players.get(entity).is_ok() {
            commands
                .entity(entity)
                .insert(AgentAnimationPlayer { agent_id });
            info!("Tagged animation player {:?} for {:?}", entity, agent_id);
        }

        if let Ok(child_entities) = children.get(entity) {
            pending.extend(child_entities.iter().copied());
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
        println!("HELLO PENDING UPDATE ANIMATION event");
        for (player_entity, mut player, agent_animation) in players.iter_mut() {
            if agent_animation.agent_id == *agent_id {
                let Some(gltf) = gltf_assets.get(&animation_data.gltf_handle) else {
                    warn!("GLTF not loaded yet: {:?}", animation_data.gltf_handle);
                    continue;
                };

                let Some(anim_handle) = gltf.animations.first() else {
                    warn!("No animations in GLTF: {:?}", animation_data.gltf_handle);
                    continue;
                };

                let (graph, node_index) = AnimationGraph::from_clip(anim_handle.clone());
                let graph_handle = graphs.add(graph);

                commands
                    .entity(player_entity)
                    .insert(AnimationGraphHandle(graph_handle));
                player.play(node_index).repeat();

                info!(
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

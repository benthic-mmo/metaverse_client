use crate::render::{AgentEntity, AgentIDMap};
use bevy::{
    animation::{
        graph::{AnimationGraph, AnimationGraphHandle},
        AnimationPlayer,
    },
    asset::{AssetServer, Assets, Handle},
    ecs::{
        entity::Entity,
        hierarchy::Children,
        name::Name,
        resource::Resource,
        system::{Commands, Query, ResMut},
    },
    platform::collections::HashMap,
    state::commands,
};
use bevy_egui::egui::TextBuffer;
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
    mut agent_id_map: ResMut<AgentIDMap>,
    asset_server: ResMut<AssetServer>,
    mut players: Query<&mut AnimationPlayer>,
    mut graphs: ResMut<Assets<AnimationGraph>>,
    gltf_assets: ResMut<Assets<Gltf>>,
    names: Query<&Name>,
    children: Query<&Children>,
    mut commands: Commands,
    names2: Query<(Entity, &Name)>,
) {
    let mut done = vec![];
    for (agent_id, animation_data) in animation_queue.pending.iter() {
        if let Some(agent) = agent_id_map.entities.get_mut(agent_id) {
            println!(
                "Checking animation for agent {:?}, entity {:?}",
                agent_id, agent.entity
            );
            if asset_server
                .load_state(&animation_data.gltf_handle)
                .is_loaded()
            {
                if let Some(gltf) = gltf_assets.get(&animation_data.gltf_handle) {
                    println!(
                        "Animation file loaded with {} animations",
                        gltf.animations.len()
                    );
                    println!(
                        "Animation bones: {:?}",
                        gltf.named_nodes.keys().collect::<Vec<_>>()
                    );

                    check_animation_player(names2, &players);
                    if let Some(anim_handle) = gltf.animations.first() {
                        println!("Using animation clip: {:?}", anim_handle);
                        print_skeleton(agent.entity, &names, &children);
                        if let Some(skeleton) =
                            print_agent_bones(agent, names, children, &mut commands)
                        {
                            agent.skeleton_root = Some(skeleton);
                            if let Ok(mut player) = players.get_mut(skeleton) {
                                println!("Animation player found on entity {:?}", skeleton);
                                let (graph, node_index) =
                                    AnimationGraph::from_clip(anim_handle.clone());
                                let graph_handle = graphs.add(graph);
                                commands
                                    .entity(skeleton)
                                    .insert(AnimationGraphHandle(graph_handle));

                                println!("AnimationGraphHandle added to entity {:?}", skeleton);
                                player.start(node_index);
                                println!("Animation started on node {:?}", node_index);

                                done.push(*agent_id);
                            } else {
                                println!("No AnimationPlayer found on entity {:?}", skeleton);
                            }
                        }
                    } else {
                        println!("No animations found in GLTF file");
                    }
                } else {
                    println!(
                        "GLTF asset not loaded for handle {:?}",
                        animation_data.gltf_handle
                    );
                }
            } else {
                println!(
                    "Animation asset not loaded yet for handle {:?}",
                    animation_data.gltf_handle
                );
            }
        } else {
            //println!("No agent found for ID {:?}", agent_id);
        }
    }
    for id in done {
        animation_queue.pending.remove(&id);
    }
}

fn print_agent_bones(
    agent: &AgentEntity,
    names: Query<&Name>,
    children: Query<&Children>,
    commands: &mut Commands,
) -> Option<Entity> {
    println!("Animation bones for entity {:?}:", agent.entity);

    fn recurse(
        entity: Entity,
        names: &Query<&Name>,
        children: &Query<&Children>,
        commands: &mut Commands,
    ) -> Option<Entity> {
        if let Ok(name) = names.get(entity) {
            if name.as_str() == "SkeletonRoot" {
                println!(
                    "SKELETON ROOT FOUND !!!!!!!!!!!!!!!!!!!!!!!!!!!!!! {:?}",
                    entity
                );

                commands.entity(entity).insert(AnimationPlayer::default());
                return Some(entity);
            }
        }

        if let Ok(kids) = children.get(entity) {
            for &child in kids.iter() {
                if let Some(found) = recurse(child, names, children, commands) {
                    return Some(found); // propagate up the first found
                }
            }
        }

        None // not found
    }

    recurse(agent.entity, &names, &children, commands)
}

fn check_animation_player(
    names: Query<(Entity, &Name)>,
    animation_players: &Query<&mut AnimationPlayer>,
) {
    for (entity, name) in names.iter() {
        if name.as_str() == "SkeletonRoot" {
            if let Ok(_player) = animation_players.get(entity) {
                println!("AnimationPlayer exists on SkeletonRoot!");
            } else {
                println!("No AnimationPlayer found on SkeletonRoot.");
            }
        }
    }
}

fn print_skeleton(entity: Entity, names: &Query<&Name>, children: &Query<&Children>) {
    fn recurse(e: Entity, depth: usize, names: &Query<&Name>, children: &Query<&Children>) {
        if let Ok(name) = names.get(e) {
            println!("{}{}", "  ".repeat(depth), name.as_str());
        }
        if let Ok(kids) = children.get(e) {
            for &c in kids.iter() {
                recurse(c, depth + 1, names, children);
            }
        }
    }
    recurse(entity, 0, names, children);
}

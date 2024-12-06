use std::{
    collections::VecDeque,
    sync::{Arc, Mutex},
};

use bevy::prelude::*;
use metaverse_messages::models::{client_update_data::ClientUpdateData, packet::Packet};
use tokio::sync::Notify;

#[derive(Resource)]
pub struct Notification(pub Arc<Notify>);

#[derive(Resource)]
pub struct UpdateStream(pub Arc<Mutex<Vec<ClientUpdateData>>>);

#[derive(Resource)]
pub struct ClientActionStream(pub Arc<Mutex<VecDeque<Packet>>>);

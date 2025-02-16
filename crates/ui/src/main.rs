mod chat;
mod login;
mod utils;

use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts, EguiPlugin};
use metaverse_messages::models::client_update_data::ClientUpdateData;
use std::{
    collections::VecDeque,
    sync::{Arc, Mutex},
};
use tokio::sync::Notify;

fn main() {
    let update_stream = Arc::new(Mutex::new(Vec::new()));
    let client_action_stream = Arc::new(Mutex::new(VecDeque::new()));
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(EguiPlugin)
        .insert_resource(login::LoginState::default())
        .add_systems(Startup, configure_visuals_system)
        .add_systems(Startup, login::configure_ui_state_system)
        .add_systems(Update, login::ui_login_system)
        .insert_resource(utils::UpdateStream(update_stream.clone()))
        .insert_resource(utils::ClientActionStream(client_action_stream.clone()))
        .insert_resource(utils::Notification(Arc::new(Notify::new())))
        .insert_resource(Events::<login::ClientEvent>::default())
        // on every frame check if there is an updated added to the update stream
        .add_systems(Update, check_updates)
        .add_systems(Update, chat::send_chat_message)
        .run();
}

// every frame, bevy checks for updates on the updateStream
fn check_updates(stream: Res<utils::UpdateStream>) {
    let mut stream = stream.0.lock().unwrap();
    if !stream.is_empty() {
        for update in stream.drain(..) {
            match update {
                ClientUpdateData::Packet(packet) => {
                    println!("Packet received: {:?}", packet);
                }
                ClientUpdateData::String(string) => {
                    println!("String received: {:?}", string)
                }
                ClientUpdateData::Error(error) => {
                    println!("Error received: {:?}", error);
                }
                ClientUpdateData::LoginProgress(login) => {
                    println!("Login Progress received {:?}", login)
                }
                ClientUpdateData::ChatFromSimulator(chat) => {
                    println!("Chat received {:?}", chat)
                }
            }
        }
    }
}

fn configure_visuals_system(mut contexts: EguiContexts) {
    contexts.ctx_mut().set_visuals(egui::Visuals {
        window_rounding: 0.0.into(),
        ..Default::default()
    });
}


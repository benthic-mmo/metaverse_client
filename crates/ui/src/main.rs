mod chat;
mod login;
mod utils;

use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts, EguiPlugin, EguiSettings};
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
        .add_systems(Update, update_ui_scale_factor_system)
        .add_systems(Update, login::ui_example_system)
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

fn update_ui_scale_factor_system(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut toggle_scale_factor: Local<Option<bool>>,
    mut contexts: Query<(&mut EguiSettings, &Window)>,
) {
    if keyboard_input.just_pressed(KeyCode::Slash) || toggle_scale_factor.is_none() {
        *toggle_scale_factor = Some(!toggle_scale_factor.unwrap_or(true));

        if let Ok((mut egui_settings, window)) = contexts.get_single_mut() {
            let scale_factor = if toggle_scale_factor.unwrap() {
                1.0
            } else {
                1.0 / window.scale_factor()
            };
            egui_settings.scale_factor = scale_factor;
        }
    }
}

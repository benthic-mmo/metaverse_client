mod login;
mod utils;
mod session;
mod chat;

use bevy::prelude::*;
use metaverse_messages::models::client_update_data::ClientUpdateData;
use std::{
    collections::VecDeque, sync::{Arc, Mutex}
};
use tokio::sync::Notify;

fn main() {
    let update_stream = Arc::new(Mutex::new(Vec::new()));
    let client_action_stream = Arc::new(Mutex::new(VecDeque::new()));
    App::new()
        .add_plugins(DefaultPlugins)
        .insert_resource(utils::UpdateStream(update_stream.clone()))
        .insert_resource(utils::ClientActionStream(client_action_stream.clone()))
        .insert_resource(utils::Notification(Arc::new(Notify::new())))
        .insert_resource(Events::<login::ClientEvent>::default())


        .add_systems(Startup, session::setup_actix)
        .add_systems(Startup, login::setup_login_ui)
        // on every frame check if there is an updated added to the update stream
        .add_systems(Update, check_updates)
        .add_systems(Update, chat::send_chat_message)
        .add_systems(Update, login::button_system)
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



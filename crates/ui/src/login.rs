use actix_rt::System;
use std::thread;

use crate::utils;

use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};
use metaverse_login::models::simulator_login_protocol::Login;
use metaverse_messages::models::{
    chat_from_viewer::{ChatFromViewer, ClientChatType},
    client_update_data::ClientUpdateData,
    packet::Packet,
};
use metaverse_session::session::Session;

#[derive(Event)]
pub struct ClientEvent(pub Packet);

#[derive(Default, Resource, Clone)]
pub struct LoginState {
    first_name: String,
    last_name: String,
    password: String,
    grid: String,
    is_window_open: bool,
}

pub fn configure_ui_state_system(mut ui_state: ResMut<LoginState>) {
    ui_state.is_window_open = true;
}

pub fn ui_example_system(
    mut login_state: ResMut<LoginState>,
    mut is_initialized: Local<bool>,
    mut contexts: EguiContexts,
    stream: ResMut<utils::UpdateStream>,
    client_action_stream: ResMut<utils::ClientActionStream>,
) {
    if !*is_initialized {
        *is_initialized = true;
    }

    let mut login = false;
    let ctx = contexts.ctx_mut();

    egui::SidePanel::left("side_panel")
        .default_width(200.0)
        .show(ctx, |ui| {
            ui.heading("Side Panel");

            ui.horizontal(|ui| {
                ui.label("first name: ");
                ui.text_edit_singleline(&mut login_state.first_name);
            });

            ui.horizontal(|ui| {
                ui.label("last name: ");
                ui.text_edit_singleline(&mut login_state.last_name);
            });
            ui.horizontal(|ui| {
                ui.label("Password: ");
                ui.text_edit_singleline(&mut login_state.password);
            });

            ui.horizontal(|ui| {
                ui.label("Grid: ");
                ui.text_edit_singleline(&mut login_state.grid);
            });

            ui.allocate_space(egui::Vec2::new(1.0, 100.0));
            ui.horizontal(|ui| {
                login = ui.button("Login").clicked();
            });
        });

    // if the login button is pressed, initiate the session!
    if login {
        // this may be more idiomatic if I made this into an event
        // HOWEVER, I do not care right now this works :) 
        init_session(stream, client_action_stream, login_state)
    }
}

fn init_session(
    stream: ResMut<utils::UpdateStream>,
    client_action_stream: ResMut<utils::ClientActionStream>,
    login_state: ResMut<LoginState>,
) {
    let stream_clone = stream.0.clone();
    let client_action_stream_clone = client_action_stream.0.clone();

    // I need to say 100 hail marys after writing this
    // someone smarter than me please help
    let login_state_clone = login_state.clone();
    thread::spawn(move || {
        let system = System::new();
        system.block_on(async {
            let login_state_clone_clone = login_state_clone.clone();
            let result = Session::new(
                Login {
                    first: login_state_clone_clone.first_name,
                    last: login_state_clone_clone.last_name,
                    passwd: login_state_clone_clone.password,
                    channel: "benthic".to_string(),
                    start: "home".to_string(),
                    agree_to_tos: true,
                    read_critical: true,
                },
                build_url("http://127.0.0.1", 9000),
                stream_clone.clone(),
            )
            .await;
            match result {
                Ok(s) => {
                    println!("successfully logged in");
                    // run forever
                    loop {
                        let mut client_stream = client_action_stream_clone.lock().unwrap();
                        if let Some(packet) = client_stream.pop_front() {
                            if let Err(e) = s.mailbox.send(packet).await {
                                eprintln!("Failed to send packet {:?}", e);
                            }
                        }
                    }
                }
                Err(e) => {
                    println!("{}", e)
                }
            }
        });
    });
}

fn build_url(url: &str, port: u16) -> String {
    let mut url_string = "".to_owned();
    url_string.push_str(url);
    url_string.push(':');
    url_string.push_str(&port.to_string());
    println!("url string {}", url_string);
    url_string
}

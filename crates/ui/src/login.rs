use std::{
    error::Error,
    sync::{Arc, Mutex},
};

use crate::utils::Notification;

use bevy::prelude::*;
use metaverse_login::models::simulator_login_protocol::Login;
use metaverse_messages::models::{
    chat_from_viewer::{ChatFromViewer, ClientChatType},
    client_update_data::ClientUpdateData,
    packet::Packet,
};
use metaverse_session::session::Session;
use bevy_egui::{egui, EguiContexts};

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
    notify: Res<Notification>,
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

    if login {
        notify.0.notify_one();
    }
}

pub async fn create_session<'a>(
    stream: Arc<Mutex<Vec<ClientUpdateData>>>,
    login_state: LoginState,
) -> Result<Session, Box<dyn Error>> {
    println!("FIRST NAME {}", login_state.first_name);
    println!("LAST NAME {}", login_state.last_name);
    println!("GRID {}", login_state.grid);
    println!("PASSWORD {}", login_state.password);
    let result = Session::new(
        Login {
            first: login_state.first_name,
            last: login_state.last_name,
            passwd: login_state.password,
            channel: "benthic".to_string(),
            start: "home".to_string(),
            agree_to_tos: true,
            read_critical: true,
        },
        build_url("http://127.0.0.1", 9000),
        stream.clone(),
    )
    .await;
    match result {
        Ok(s) => Ok(s),
        Err(e) => Err(Box::new(e)),
    }
}

fn build_url(url: &str, port: u16) -> String {
    let mut url_string = "".to_owned();
    url_string.push_str(url);
    url_string.push(':');
    url_string.push_str(&port.to_string());
    println!("url string {}", url_string);
    url_string
}

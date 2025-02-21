use std::os::unix::net::UnixDatagram;

use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};
use metaverse_messages::{login_system::login::Login, packet::Packet};

use crate::Sockets;

#[derive(Default, Resource, Clone)]
pub struct LoginData {
    first_name: String,
    last_name: String,
    password: String,
    grid: String,
}

pub fn ui_login_system(
    mut login_data: ResMut<LoginData>,
    mut is_initialized: Local<bool>,
    mut contexts: EguiContexts,
    sockets: Res<Sockets>,
) {
    if !*is_initialized {
        *is_initialized = true;
    }

    let mut login = false;
    let ctx = contexts.ctx_mut();

    egui::SidePanel::left("Login")
        .default_width(200.0)
        .show(ctx, |ui| {
            ui.heading("Side Panel");

            ui.horizontal(|ui| {
                ui.label("First Name: ");
                ui.text_edit_singleline(&mut login_data.first_name);
            });

            ui.horizontal(|ui| {
                ui.label("Last Name: ");
                ui.text_edit_singleline(&mut login_data.last_name);
            });
            ui.horizontal(|ui| {
                ui.label("Password: ");
                ui.text_edit_singleline(&mut login_data.password);
            });

            ui.horizontal(|ui| {
                ui.label("Grid: ");
                ui.text_edit_singleline(&mut login_data.grid);
            });

            ui.allocate_space(egui::Vec2::new(1.0, 100.0));
            ui.horizontal(|ui| {
                login = ui.button("Login").clicked();
            });
        });
    if login {
        let grid = if login_data.grid == "localhost" {
            build_url("http://127.0.0.1", 9000)
        } else {
            // handle the URL normally
            "http".to_string()
        };

        let packet = Packet::new_login_packet(Login {
            first: login_data.first_name.clone(),
            last: login_data.last_name.clone(),
            passwd: login_data.password.clone(),
            start: "home".to_string(),
            channel: "benthic".to_string(),
            agree_to_tos: true,
            read_critical: true,
            url: grid,
        })
        .to_bytes();

        let client_socket = UnixDatagram::unbound().unwrap();
        match client_socket.send_to(&packet, &sockets.ui_to_server_socket) {
            Ok(_) => println!("Login sent from UI"),
            Err(e) => println!("Error sending login from UI {:?}", e),
        };
    }
}

fn build_url(url: &str, port: u16) -> String {
    let mut url_string = "".to_owned();
    url_string.push_str(url);
    url_string.push(':');
    url_string.push_str(&port.to_string());
    url_string
}

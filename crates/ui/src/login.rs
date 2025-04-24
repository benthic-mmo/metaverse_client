use crate::{Sockets, ViewerState};
use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};
use metaverse_messages::{login_system::login::Login, packet::Packet};
use std::net::UdpSocket;

#[derive(Default, Resource, Clone)]
pub struct LoginData {
    first_name: String,
    last_name: String,
    password: String,
    grid: String,
}

pub fn login_screen(
    mut login_data: ResMut<LoginData>,
    mut is_initialized: Local<bool>,
    mut contexts: EguiContexts,
    mut viewer_state: ResMut<NextState<ViewerState>>,
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
        // display loading screen after login
        viewer_state.set(ViewerState::Loading);

        let grid = if login_data.grid == "localhost" {
            format!("{}:{}", "http://127.0.0.1", 9000)
        } else {
            format!("http://{}", login_data.grid)
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
        let client_socket = UdpSocket::bind("0.0.0.0:0").unwrap();
        match client_socket.send_to(
            &packet,
            format!("127.0.0.1:{}", &sockets.ui_to_server_socket),
        ) {
            Ok(_) => println!("Login sent from UI"),
            Err(e) => println!("Error sending login from UI {:?}", e),
        };
    }
}

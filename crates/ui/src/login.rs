use crate::{CONFIG_FILE, ShareDir, Sockets, VIEWER_NAME, ViewerState};
use bevy::prelude::*;
use bevy_egui::{EguiContexts, egui};
use keyring::Entry;
use metaverse_messages::{login::login_xmlrpc::Login, packet::packet::Packet};
use std::{fs, net::UdpSocket, path::PathBuf};

#[derive(Default, Resource, Clone)]
pub struct LoginData {
    pub first_name: String,
    pub last_name: String,
    pub password: String,
    pub grid: String,
    pub remember_me: bool,
}

pub fn login_screen(
    mut login_data: ResMut<LoginData>,
    mut is_initialized: Local<bool>,
    mut contexts: EguiContexts,
    mut viewer_state: ResMut<NextState<ViewerState>>,
    sockets: Res<Sockets>,
    share_dir: ResMut<ShareDir>,
) {
    if !*is_initialized {
        *is_initialized = true;
    }

    let mut login = false;
    let ctx = contexts.ctx_mut();

    egui::SidePanel::left("Login")
        .default_width(200.0)
        .show(ctx, |ui| {
            ui.heading("Login");

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

            ui.horizontal(|ui| {
                ui.checkbox(&mut login_data.remember_me, "Remember Me");
            });

            ui.allocate_space(egui::Vec2::new(1.0, 100.0));
            ui.horizontal(|ui| {
                login = ui.button("Login").clicked();
            });
        });
    if login {
        // display loading screen after login
        viewer_state.set(ViewerState::Loading);

        let path = PathBuf::from(CONFIG_FILE);
        let username = format!(
            "{} {} {}",
            login_data.first_name, login_data.last_name, login_data.grid
        );

        if login_data.remember_me {
            println!("{}", username);
            match Entry::new(VIEWER_NAME, &username) {
                Ok(entry) => {
                    if let Err(e) = entry.set_password(&login_data.password) {
                        eprintln!("failed to store password {:?}", e)
                    }
                }
                Err(e) => println!("failed to create new keyring entry {:?}", e),
            }
            if let Some(file_path) = &share_dir.path {
                if let Err(e) = fs::write(file_path, &username) {
                    eprintln!("Error writing file: {}", e);
                }
            }
        } else {
            if let Err(e) =
                Entry::new(VIEWER_NAME, &username).and_then(|entry| entry.delete_credential())
            {
                eprintln!("Failed to store password: {:?}", e);
            }
            if let Err(e) = fs::remove_file(path) {
                eprintln!("Failed to remove stored data {:?}", e)
            };
        }

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

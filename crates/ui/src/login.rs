use crate::{
    errors::{CredentialDeleteError, CredentialLoadError, CredentialStoreError, PacketSendError},
    login,
    plugin::{send_packet_to_core, ShareDir, Sockets, ViewerState, VIEWER_NAME},
};
use bevy::log::error;
use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};
use keyring::Entry;
use metaverse_messages::{
    login::login_xmlrpc::Login,
    packet::message::{EventType, UiMessage},
};
use std::{fs, path::PathBuf};

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
    viewer_state: ResMut<NextState<ViewerState>>,
    sockets: Res<Sockets>,
    share_dir: ResMut<ShareDir>,
) -> Result {
    if !*is_initialized {
        *is_initialized = true;
    }

    let mut login = false;
    let ctx = contexts.ctx_mut()?;
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
    if login
        && let Err(e) = send_login(viewer_state, login_data, share_dir, sockets) {
            error!("{:?}", e)
        };
    Ok(())
}

fn send_login(
    mut viewer_state: ResMut<NextState<ViewerState>>,
    login_data: ResMut<LoginData>,
    share_dir: ResMut<ShareDir>,
    sockets: Res<Sockets>,
) -> Result<(), PacketSendError> {
    // display loading screen after login
    viewer_state.set(ViewerState::Loading);

    let username = format!(
        "{} {} {}",
        login_data.first_name, login_data.last_name, login_data.grid
    );

    if login_data.remember_me {
        if let Err(e) = store_creds(&username, &login_data.password, &share_dir.login_cred_path) {
            warn!("{:?}", e)
        }
    } else if let Err(e) = delete_creds(&username, &share_dir.login_cred_path) {
        warn!("{:?}", e)
    }

    let grid = if login_data.grid == "localhost" {
        format!("{}:{}", "http://127.0.0.1", 9000)
    } else {
        format!("http://{}", login_data.grid)
    };

    let packet = UiMessage::from_event(&EventType::new_login_event(Login {
        first: login_data.first_name.clone(),
        last: login_data.last_name.clone(),
        passwd: login_data.password.clone(),
        start: "last".to_string(),
        channel: VIEWER_NAME.to_string(),
        agree_to_tos: true,
        read_critical: true,
        url: grid,
    }))
    .to_bytes();
    send_packet_to_core(&packet, &sockets)?;
    Ok(())
}

fn store_creds(
    username: &str,
    password: &str,
    cred_path: &PathBuf,
) -> Result<(), CredentialStoreError> {
    let entry = Entry::new(VIEWER_NAME, username)?;
    entry.set_password(password)?;
    fs::write(cred_path, username)?;
    Ok(())
}

fn delete_creds(username: &str, cred_path: &PathBuf) -> Result<(), CredentialDeleteError> {
    let entry = Entry::new(VIEWER_NAME, username)?;
    entry.delete_credential()?;
    fs::remove_file(cred_path)?;
    Ok(())
}

// load the login data from the keyring. Return a CredentialLoadError if it fails.
pub fn load_login_data(file_path: &PathBuf) -> Result<LoginData, CredentialLoadError> {
    let content = fs::read_to_string(file_path)?;
    let keyring = Entry::new(VIEWER_NAME, &content)?;
    let password = keyring.get_password()?;

    let mut parts = content.split_whitespace();
    let first_name = parts.next().unwrap_or_default().to_string();
    let last_name = parts.next().unwrap_or_default().to_string();
    let grid = parts.next().unwrap_or_default().to_string();
    Ok(login::LoginData {
        first_name,
        last_name,
        grid,
        remember_me: true,
        password,
    })
}

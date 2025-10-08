use bevy::app::TerminalCtrlCHandlerPlugin;
use bevy::asset::UnapprovedPathMode;
use bevy::prelude::*;
use bevy_egui::EguiPlugin;
use bevy_egui::EguiPrimaryContextPass;
use bevy_panorbit_camera::PanOrbitCameraPlugin;
use ui::chat::chat_screen;
use ui::loading::loading_screen;
use ui::login::login_screen;
use ui::plugin::MetaversePlugin;
use ui::plugin::ViewerState;

pub const CONFIG_FILE: &str = "login_conf.json";

fn main() {
    App::new()
        .add_plugins(
            DefaultPlugins
                .set(AssetPlugin {
                    file_path: "assets".into(),
                    unapproved_path_mode: UnapprovedPathMode::Allow,
                    ..default()
                })
                .set(WindowPlugin {
                    close_when_requested: false,
                    ..default()
                })
                .set(TerminalCtrlCHandlerPlugin {}),
        )
        .add_plugins(PanOrbitCameraPlugin)
        .add_plugins(EguiPlugin::default())
        .add_plugins(MetaversePlugin)
        .add_systems(
            EguiPrimaryContextPass,
            login_screen.run_if(in_state(ViewerState::Login)),
        )
        .add_systems(
            EguiPrimaryContextPass,
            loading_screen.run_if(in_state(ViewerState::Loading)),
        )
        .add_systems(
            EguiPrimaryContextPass,
            chat_screen.run_if(in_state(ViewerState::Chat)),
        )
        .run();
}

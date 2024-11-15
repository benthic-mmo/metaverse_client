use std::{error::Error, sync::{Arc, Mutex}};

use crate::utils::Notification;

use bevy::{color::palettes::basic::*, prelude::*};
use metaverse_login::models::simulator_login_protocol::Login;
use metaverse_messages::models::{chat_from_viewer::{ChatFromViewer, ClientChatType}, client_update_data::ClientUpdateData, packet::Packet};
use metaverse_session::session::Session;
use uuid::Uuid;
const NORMAL_BUTTON: Color = Color::srgb(0.15, 0.15, 0.15);
const HOVERED_BUTTON: Color = Color::srgb(0.25, 0.25, 0.25);
const PRESSED_BUTTON: Color = Color::srgb(0.35, 0.75, 0.35);

#[derive(Event)]
pub struct ClientEvent(pub Packet);

pub fn setup_login_ui(
    mut commands: Commands,
    asset_server: Res<AssetServer>
){
    commands.spawn(Camera2dBundle::default());
    commands
        .spawn(NodeBundle {
            style: Style {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
            ..default()
        })
        .with_children(|parent| {
            parent
                .spawn(ButtonBundle {
                    style: Style {
                        width: Val::Px(150.0),
                        height: Val::Px(65.0),
                        border: UiRect::all(Val::Px(5.0)),
                        // horizontally center child text
                        justify_content: JustifyContent::Center,
                        // vertically center child text
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    border_color: BorderColor(Color::BLACK),
                    border_radius: BorderRadius::MAX,
                    background_color: NORMAL_BUTTON.into(),
                    ..default()
                })
                .with_children(|parent| {
                    parent.spawn(TextBundle::from_section(
                        "Button",
                        TextStyle {
                            font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                            font_size: 40.0,
                            color: Color::srgb(0.9, 0.9, 0.9),
                        },
                    ));
                });
        });
}

pub fn button_system(
    mut interaction_query: Query<
        (
            &Interaction,
            &mut BackgroundColor,
            &mut BorderColor,
            &Children,
        ),
        (Changed<Interaction>, With<Button>),
    >,
    mut text_query: Query<&mut Text>,
    notify: Res<Notification>,
    mut ev_press: EventWriter<ClientEvent>,
) {
    for (interaction, mut color, mut border_color, children) in &mut interaction_query {
        let mut text = text_query.get_mut(children[0]).unwrap();
        match *interaction {
            Interaction::Pressed => {
                text.sections[0].value = "Press".to_string();
                *color = PRESSED_BUTTON.into();
                border_color.0 = RED.into();
                // login button was pressed, notify the actix thread to login
                notify.0.notify_one();

                ev_press.send(ClientEvent(Packet::new_chat_from_viewer(ChatFromViewer {
                    agent_id: Uuid::nil(),   // set these to nil and set them properly later
                    session_id: Uuid::nil(), // set these to nil and set them properly later
                    message: "hello world".to_string(),
                    message_type: ClientChatType::Normal,
                    channel: 0,
                })));
                println!("pressed");
            }
            Interaction::Hovered => {
                text.sections[0].value = "Hover".to_string();
                *color = HOVERED_BUTTON.into();
                border_color.0 = Color::WHITE;
            }
            Interaction::None => {
                text.sections[0].value = "Button".to_string();
                *color = NORMAL_BUTTON.into();
                border_color.0 = Color::BLACK;
            }
        }
    }
}

pub async fn create_session(stream: Arc<Mutex<Vec<ClientUpdateData>>>) -> Result<Session, Box<dyn Error>> {
    let (first_name, last_name, password, grid) = get_user_login();
    let result = Session::new(
        Login {            
            first: first_name.to_string(),
            last: last_name.to_string(),
            passwd: password.to_string(),
            channel: "benthic".to_string(),
            start: "home".to_string(),
            agree_to_tos: true,
            read_critical: true,
        },
        grid,
        stream.clone(),
    )
    .await;
    match result {
        Ok(s) => Ok(s),
        Err(e) => Err(Box::new(e)),
    }
}


fn get_user_login() -> (String, String, String, String) {
    let first_name = "default";
    let last_name = "user";
    let password = "password";
    let grid = build_url(&"http://127.0.0.1", 9000);
    return (
        first_name.to_string(),
        last_name.to_string(),
        password.to_string(),
        grid.to_string(),
    );
}

fn build_url(url: &str, port: u16) -> String {
    let mut url_string = "".to_owned();
    url_string.push_str(url);
    url_string.push(':');
    url_string.push_str(&port.to_string());
    println!("url string {}", url_string);
    url_string
}


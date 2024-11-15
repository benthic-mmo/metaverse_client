use actix_rt::System;
use bevy::{color::palettes::basic::*, prelude::*, tasks::IoTaskPool};
use metaverse_login::models::simulator_login_protocol::Login;
use metaverse_messages::models::{
    chat_from_viewer::{ChatFromViewer, ClientChatType},
    client_update_data::ClientUpdateData,
    packet::Packet,
};
use metaverse_session::session::Session;
use std::{
    collections::VecDeque, error::Error, sync::{Arc, Mutex}, thread
};
use tokio::sync::Notify;
use uuid::Uuid;

#[derive(Resource)]
struct UpdateStream(Arc<Mutex<Vec<ClientUpdateData>>>);

#[derive(Resource)]
struct ClientActionStream(Arc<Mutex<VecDeque<Packet>>>);

#[derive(Resource)]
struct Notification(Arc<Notify>);

#[derive(Event)]
struct ClientEvent(Packet);

fn main() {
    let update_stream = Arc::new(Mutex::new(Vec::new()));
    let client_action_stream = Arc::new(Mutex::new(VecDeque::new()));
    App::new()
        .add_plugins(DefaultPlugins)
        .insert_resource(UpdateStream(update_stream.clone()))
        .insert_resource(ClientActionStream(client_action_stream.clone()))
        .insert_resource(Notification(Arc::new(Notify::new())))
        .insert_resource(Events::<ClientEvent>::default())
        .add_systems(Startup, setup)
        // on every frame check if there is an updated added to the update stream
        .add_systems(Update, check_updates)
        .add_systems(Update, send_chat_message)
        .add_systems(Update, button_system)
        .add_systems(Update, heartbeat)
        .run();
}

fn heartbeat(){
    print!(".");
}

fn send_chat_message(
    mut ev_client_event: EventReader<ClientEvent>,
    client_action_stream: ResMut<ClientActionStream>,
) {
    for event in ev_client_event.read() {
        let mut client_action_stream = client_action_stream.0.lock().unwrap();
        client_action_stream.push_back(event.0.clone());
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

async fn login(stream: Arc<Mutex<Vec<ClientUpdateData>>>) -> Result<Session, Box<dyn Error>> {
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

// every frame, bevy checks for updates on the updateStream
fn check_updates(stream: Res<UpdateStream>) {
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
    //println!("Checking updates")
}

const NORMAL_BUTTON: Color = Color::srgb(0.15, 0.15, 0.15);
const HOVERED_BUTTON: Color = Color::srgb(0.25, 0.25, 0.25);
const PRESSED_BUTTON: Color = Color::srgb(0.35, 0.75, 0.35);

fn button_system(
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

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    notify: Res<Notification>,
    stream: ResMut<UpdateStream>,
    client_action_stream: ResMut<ClientActionStream>,
) {
    let notify_clone = notify.0.clone();
    let stream_clone = stream.0.clone();
    let client_action_stream_clone = client_action_stream.0.clone();

    let task_pool = IoTaskPool::get();

    task_pool
        .spawn(async move {
            println!("Task pool has started!");
        })
        .is_finished();

    // I need to say 100 hail marys after writing this
    // someone smarter than me please help
    thread::spawn(move || {
        let system = System::new();
        system.block_on(async {
            // The loop is for the login, to retry until it succeeds
            loop {
                // block the thread until it has been notified that the login button has been
                // pressed
                notify_clone.notified().await;
                // login and create the session
                let result = login(stream_clone.clone()).await;
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
            }
            // keep the thread alive for the lifetime of the app and handle messages sent to
            // mailbox
        });
    })
    .is_finished();

    // ui camera
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

use actix::System;
use metaverse_login::models::simulator_login_protocol::Login;
use metaverse_messages::models::chat_from_viewer::{ChatFromViewer, ClientChatType};
use metaverse_messages::models::client_update_data::ClientUpdateData;
use metaverse_messages::models::packet::Packet;
use metaverse_session::session::Session;
use std::io::{self, Write};
use std::sync::{Arc, Mutex};
use tokio::time::{sleep, Duration};

fn get_user_login() -> (String, String, String, String) {
    print!("First Name:");
    io::stdout().flush().unwrap();
    let mut first_name = String::new();
    io::stdin()
        .read_line(&mut first_name)
        .expect("Failed to read line");
    let first_name = first_name.trim();

    print!("Last Name:");
    io::stdout().flush().unwrap();
    let mut last_name = String::new();
    io::stdin()
        .read_line(&mut last_name)
        .expect("Failed to read line");
    let last_name = last_name.trim();

    print!("Password: ");
    io::stdout().flush().unwrap();
    let mut password = String::new();
    io::stdin()
        .read_line(&mut password)
        .expect("Failed to read line");
    let password = password.trim();

    print!("Grid: ");
    io::stdout().flush().unwrap();
    let mut grid = String::new();
    io::stdin()
        .read_line(&mut grid)
        .expect("Failed to read line");
    let grid = grid.trim();

    return (
        first_name.to_string(),
        last_name.to_string(),
        password.to_string(),
        grid.to_string(),
    );
}

fn main() {
    let runtime = System::new();
    let update_stream = Arc::new(Mutex::new(Vec::new()));
    let update_stream_clone = update_stream.clone();
    let update_stream_clone_2 = update_stream.clone();

    runtime.block_on(async {
        tokio::spawn(async move { check_for_updates(update_stream_clone_2).await });
        let session;
        loop {
            let (first_name, last_name, password, grid) = get_user_login();
            let grid = if grid == "localhost" {
                "http://127.0.0.1".to_string()
            } else {
                grid.to_string()
            };

            let grid = build_url(&grid, 9000);

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
                update_stream_clone.clone(),
            )
            .await;
            match result {
                Ok(s) => {
                    session = s;
                    break;
                }
                Err(_) => {
                    println!("Login failed");
                }
            }
        }

        loop {
            print!("Chat: ");
            io::stdout().flush().unwrap(); // Ensure the prompt is displayed before input

            let mut input = String::new();
            io::stdin()
                .read_line(&mut input)
                .expect("Failed to read line");

            let input = input.trim(); // Remove any trailing newline or whitespace

            if input == "quit" {
                println!("Goodbye!");
                break; // Exit the loop if the user types "quit"
            }
            sleep(Duration::from_secs(1)).await;
            match session
                .mailbox
                .send(Packet::new_chat_from_viewer(ChatFromViewer {
                    agent_id: session.agent_id,
                    session_id: session.session_id,
                    message: input.to_string(),
                    message_type: ClientChatType::Normal,
                    channel: 0,
                }))
                .await
            {
                Ok(_) => println!("chat sent: {:?}", input),
                Err(_) => print!("chat failed to send"),
            };
        }
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

async fn check_for_updates(stream: Arc<Mutex<Vec<ClientUpdateData>>>) {
    loop {
        sleep(Duration::from_millis(10)).await;
        let mut stream = stream.lock().unwrap();
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
}

use std::thread;
use actix_rt::System;

use bevy::prelude::*;
use crate::{login, utils};

pub fn setup_actix(
    notify: Res<utils::Notification>,
    stream: ResMut<utils::UpdateStream>,
    client_action_stream: ResMut<utils::ClientActionStream>,
) {
    let notify_clone = notify.0.clone();
    let stream_clone = stream.0.clone();
    let client_action_stream_clone = client_action_stream.0.clone();

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
                let result = login::create_session(stream_clone.clone()).await;
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
        });
    });
   }



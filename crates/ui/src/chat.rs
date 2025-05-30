use std::net::UdpSocket;

use crate::{ChatMessages, SessionData, Sockets};
use bevy::ecs::system::{Res, ResMut};
use bevy::prelude::Resource;
use bevy_egui::{EguiContexts, egui};
use metaverse_messages::chat::ChatType;
use metaverse_messages::chat::chat_from_viewer::ChatFromViewer;
use metaverse_messages::packet::packet::Packet;

#[derive(Default, Resource, Clone)]
pub struct ChatMessage {
    message: String,
}

pub fn chat_screen(
    mut contexts: EguiContexts,
    mut chat_message: ResMut<ChatMessage>,
    session_data: Res<SessionData>,
    sockets: Res<Sockets>,
    chat_messages: Res<ChatMessages>,
) {
    let ctx = contexts.ctx_mut();
    let mut send = false;

    egui::Window::new("Chat")
        .default_width(300.0)
        .resizable(true)
        .collapsible(true)
        .show(ctx, |ui| {
            egui::ScrollArea::vertical()
                .stick_to_bottom(true)
                .max_height(300.0)
                .auto_shrink([false, true])
                .show(ui, |ui| {
                    ui.allocate_space(egui::vec2(ui.available_width(), 300.0));
                    for (i, message) in chat_messages.messages.iter().enumerate() {
                        ui.push_id(i, |ui| {
                            ui.label(format!("{}: {}", message.user, message.message));
                        });
                    }
                });

            ui.separator();

            // Chat input area
            ui.horizontal(|ui| {
                ui.label("Chat:");
                let text_edit_response = ui.text_edit_singleline(&mut chat_message.message);
                if text_edit_response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter))
                {
                    text_edit_response.request_focus();
                    send = true;
                }
                if send && !chat_message.message.trim().is_empty() {
                    send = true;
                }
            });
        });

    if (chat_message.message != "") && send {
        let message = chat_message.message.clone();
        chat_message.message = "".to_string();
        let data = session_data.login_response.as_ref().unwrap();
        let packet = Packet::new_chat_from_viewer(ChatFromViewer {
            session_id: data.session_id,
            agent_id: data.agent_id,
            message,
            channel: 0,
            message_type: ChatType::Normal,
        })
        .to_bytes();
        let client_socket = UdpSocket::bind("0.0.0.0:0").unwrap();
        match client_socket.send_to(
            &packet,
            format!("127.0.0.1:{}", sockets.ui_to_server_socket),
        ) {
            Ok(_) => println!("Chat message sent from UI"),
            Err(e) => println!("Error sending chat from UI {:?}", e),
        };
    }
}

use std::net::UdpSocket;

use bevy::ecs::system::{Res, ResMut, Resource};
use bevy_egui::{egui, EguiContexts};
use metaverse_messages::{
    chat_from_viewer::{ChatFromViewer, ClientChatType},
    packet::Packet,
};

use crate::{ChatMessages, SessionData, Sockets};

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
    egui::TopBottomPanel::bottom("Chat Bar").show(ctx, |ui| {
        ui.vertical(|ui| {
            egui::ScrollArea::vertical()
                .auto_shrink([false, true])
                .show(ui, |ui| {
                    for (i, message) in chat_messages.messages.iter().enumerate() {
                        ui.push_id(i, |ui| {
                            ui.label(format!(
                                "{}: {}",
                                message.user.clone(),
                                message.message.clone()
                            )); // Display the message
                        });
                    }
                });
        });
        egui::Frame::default().inner_margin(4.0).show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label("Chat:");
                ui.text_edit_singleline(&mut chat_message.message);
                send = ui.button("Send").clicked();
            });
        });
    });
    if send {
        let message = chat_message.message.clone();
        chat_message.message = "".to_string();
        let data = session_data.login_response.as_ref().unwrap();
        let packet = Packet::new_chat_from_viewer(ChatFromViewer {
            session_id: data.session_id.unwrap(),
            agent_id: data.agent_id.unwrap(),
            message,
            channel: 0,
            message_type: ClientChatType::Normal,
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

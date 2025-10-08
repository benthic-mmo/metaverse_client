use crate::errors::ChatError;
use crate::plugin::{
    retrieve_login_response, send_packet_to_core, ChatMessages, SessionData, Sockets, ViewerState,
};
use bevy::ecs::error::Result;
use bevy::ecs::system::{Res, ResMut};
use bevy::log::error;
use bevy::prelude::Resource;
use bevy::state::state::NextState;
use bevy_egui::{egui, EguiContexts};
use metaverse_messages::chat::chat_from_viewer::ChatFromViewer;
use metaverse_messages::chat::ChatType;
use metaverse_messages::packet::message::{EventType, UiMessage};

#[derive(Default, Resource, Clone)]
pub struct ChatMessage {
    message: String,
}

pub fn chat_screen(
    mut contexts: EguiContexts,
    mut chat_message: ResMut<ChatMessage>,
    viewer_state: ResMut<NextState<ViewerState>>,
    session_data: Res<SessionData>,
    sockets: Res<Sockets>,
    chat_messages: Res<ChatMessages>,
) -> Result {
    let ctx = contexts.ctx_mut()?;
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

    if (!chat_message.message.is_empty()) && send {
        if let Err(e) = send_chat(&chat_message.message, session_data, sockets, viewer_state) {
            match e {
                // if the loginresponse is not populated, return to the login screen
                ChatError::ChatLoginError(_) => return Ok(()),
                e => error!("{:?}", e),
            }
        };
        chat_message.message.clear();
    }
    Ok(())
}
fn send_chat(
    message: &str,
    session_data: Res<SessionData>,
    sockets: Res<Sockets>,
    mut viewer_state: ResMut<NextState<ViewerState>>,
) -> Result<(), ChatError> {
    let response = retrieve_login_response(&session_data, &mut viewer_state)?;
    let packet = UiMessage::from_event(&EventType::new_chat_from_viewer(ChatFromViewer {
        session_id: response.session_id,
        agent_id: response.agent_id,
        message: message.to_owned(),
        channel: 0,
        message_type: ChatType::Normal,
    }))
    .to_bytes();
    send_packet_to_core(&packet, &sockets)?;
    Ok(())
}

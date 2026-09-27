use crate::core_plugin::ChatMessageEvent;
use crate::errors::ChatError;
use crate::plugin::{ChatMessage, Sockets, ViewerState, send_packet_to_core};
use benthic_protocol::messages::ui::chat_from_viewer::ChatFromUI;
use benthic_protocol::messages::ui::ui_messages::UIResponse;
use benthic_protocol::messages::utils::chat_types::ChatType;
use bevy::app::{App, Plugin, Update};
use bevy::ecs::error::Result;
use bevy::ecs::message::MessageReader;
use bevy::ecs::resource::Resource;
use bevy::ecs::schedule::IntoScheduleConfigs;
use bevy::ecs::system::{Res, ResMut};
use bevy::log::error;
use bevy::state::condition::in_state;
use bevy_egui::{EguiContexts, EguiPrimaryContextPass, egui};

#[derive(Resource)]
pub struct ChatMessages {
    pub messages: Vec<ChatFromClientMessage>,
}

pub struct ChatFromClientMessage {
    pub user: String,
    pub message: String,
}

pub struct ChatPlugin;
impl Plugin for ChatPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(ChatMessages {
            messages: Vec::new(),
        })
        .insert_resource(ChatMessage::default())
        .add_systems(Update, handle_chat_update)
        .add_systems(
            EguiPrimaryContextPass,
            chat_panel.run_if(in_state(ViewerState::Main)),
        );
    }
}

pub fn chat_panel(
    mut contexts: EguiContexts,
    mut chat_message: ResMut<ChatMessage>,
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
        if let Err(e) = send_chat(&chat_message.message, sockets) {
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

fn handle_chat_update(
    mut events: MessageReader<ChatMessageEvent>,
    mut chat_messages: ResMut<ChatMessages>,
) {
    for event in events.read() {
        chat_messages.messages.push(ChatFromClientMessage {
            user: event.value.from_name.clone(),
            message: event.value.message.clone(),
        });
    }
}

fn send_chat(message: &str, sockets: Res<Sockets>) -> Result<(), ChatError> {
    let packet = UIResponse::new_chat_from_viewer(ChatFromUI {
        message: message.to_owned(),
        channel: 0,
        message_type: ChatType::Normal,
    })
    .to_bytes();
    send_packet_to_core(&packet, &sockets)?;
    Ok(())
}

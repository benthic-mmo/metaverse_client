use bevy::ecs::error::Result;
use bevy_egui::{EguiContexts, egui};

pub fn loading_screen(mut contexts: EguiContexts) -> Result {
    let ctx = contexts.ctx_mut()?;
    egui::CentralPanel::default().show(ctx, |ui| {
        ui.heading("Loading");
    });
    Ok(())
}

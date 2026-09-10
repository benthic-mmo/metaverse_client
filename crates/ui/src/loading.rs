use bevy::ecs::error::Result;
use bevy_egui::{EguiContexts, egui};
use egui::{LayerId, Ui, UiBuilder};

pub fn loading_screen(mut contexts: EguiContexts) -> Result {
    let ctx = contexts.ctx_mut()?;
    let mut viewport_ui = Ui::new(
        ctx.clone(),
        "viewport".into(),
        UiBuilder::new()
            .layer_id(LayerId::background())
            .max_rect(ctx.viewport_rect()),
    );
    egui::CentralPanel::default().show(&mut viewport_ui, |ui| {
        ui.heading("Loading");
    });
    Ok(())
}

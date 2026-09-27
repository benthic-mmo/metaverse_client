use crate::plugin::ViewerState;
use bevy::app::App;
use bevy::ecs::error::Result;
use bevy::prelude::*;
use bevy_egui::{EguiContexts, EguiPrimaryContextPass};

pub struct InventoryPanelPlugin;
impl Plugin for InventoryPanelPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            EguiPrimaryContextPass,
            inventory_panel.run_if(in_state(ViewerState::Main)),
        );
    }
}

pub fn inventory_panel(mut contexts: EguiContexts) -> Result {
    let ctx = contexts.ctx_mut()?;
    egui::Window::new("Inventory")
        .default_width(300.0)
        .resizable(true)
        .collapsible(true)
        .show(ctx, |_ui| egui::ScrollArea::vertical());
    Ok(())
}

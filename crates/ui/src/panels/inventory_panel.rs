use crate::{core_plugin::InventoryPopulateEvent, plugin::ViewerState};
use benthic_protocol::messages::ui::populate_inventory::{FolderContents, UiFolder};
use bevy::app::App;
use bevy::ecs::error::Result;
use bevy::prelude::*;
use bevy_egui::{EguiContexts, EguiPrimaryContextPass};

pub struct InventoryPanelPlugin;

#[derive(Resource)]
struct InventoryUi {
    folders: Vec<UiFolder>,
    default_thumbnail: Handle<Image>,
}

impl FromWorld for InventoryUi {
    fn from_world(world: &mut World) -> Self {
        let asset_server = world.resource::<AssetServer>();

        Self {
            folders: Vec::new(),
            default_thumbnail: asset_server.load("Textures/default.png"),
        }
    }
}

impl Plugin for InventoryPanelPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<InventoryUi>()
            .add_systems(
                EguiPrimaryContextPass,
                inventory_panel.run_if(in_state(ViewerState::Main)),
            )
            .add_systems(Update, handle_inventory_populate);
    }
}

fn inventory_panel(
    mut contexts: EguiContexts,
    inventory: Res<InventoryUi>,
    asset_server: Res<AssetServer>,
) -> Result {
    let mut thumbnail_ids = std::collections::HashMap::new();

    for folder in &inventory.folders {
        for contents in &folder.contents {
            if let FolderContents::Item(item) = contents {
                let thumbnail = if item.thumbnail.as_os_str().is_empty() {
                    inventory.default_thumbnail.clone()
                } else {
                    asset_server.load(item.thumbnail.clone())
                };

                let texture_id =
                    contexts.add_image(bevy_egui::EguiTextureHandle::Weak(thumbnail.id()));

                thumbnail_ids.insert(item.item_id, texture_id);
            }
        }
    }

    let ctx = contexts.ctx_mut()?;

    egui::Window::new("Inventory")
        .default_width(300.0)
        .resizable(true)
        .collapsible(true)
        .show(ctx, |ui| {
            egui::ScrollArea::vertical()
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    for folder in &inventory.folders {
                        egui::CollapsingHeader::new(&folder.name)
                            .default_open(false)
                            .show(ui, |ui| {
                                let item_width = 64.0;
                                let spacing = ui.spacing().item_spacing.x;

                                let columns =
                                    ((ui.available_width() + spacing) / (item_width + spacing))
                                        .floor()
                                        .max(1.0) as usize;

                                egui::Grid::new(folder.id)
                                    .spacing(egui::vec2(spacing, 12.0))
                                    .show(ui, |ui| {
                                        let mut column = 0;

                                        for contents in &folder.contents {
                                            if let FolderContents::Item(item) = contents {
                                                ui.vertical(|ui| {
                                                    if let Some(&texture_id) =
                                                        thumbnail_ids.get(&item.item_id)
                                                    {
                                                        ui.add_sized(
                                                            [64.0, 64.0],
                                                            egui::Image::new(
                                                                egui::load::SizedTexture::new(
                                                                    texture_id,
                                                                    [64.0, 64.0],
                                                                ),
                                                            ),
                                                        );
                                                    }

                                                    ui.label(&item.name);
                                                });

                                                column += 1;

                                                if column == columns {
                                                    ui.end_row();
                                                    column = 0;
                                                }
                                            }
                                        }

                                        if column > 0 {
                                            ui.end_row();
                                        }
                                    });
                            });
                    }
                });
        });

    Ok(())
}
fn handle_inventory_populate(
    mut events: MessageReader<InventoryPopulateEvent>,
    mut inventory: ResMut<InventoryUi>,
) {
    for event in events.read() {
        inventory.folders.push(event.value.folder.clone());
    }
}

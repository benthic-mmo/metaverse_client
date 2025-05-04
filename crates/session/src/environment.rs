use std::collections::HashMap;

use glam::U16Vec2;
use metaverse_environment::{generate_mesh::generate_land_mesh, layer_handler::Land};
use metaverse_messages::ui::custom::layer_update::LayerUpdate;

/// This handles patch generation for the land. This is waht calls the generate land mesh function
/// that outputs the gltf files. Returns a vector of LayerUpdate packets to be sent to the UI.
pub fn generate_patch(
    land: Land,
    location: U16Vec2,
    patch_queue: &mut HashMap<U16Vec2, Land>,
    total_patches: &HashMap<U16Vec2, Land>,
) -> Vec<LayerUpdate> {
    let mut completed_updates = Vec::new();
    let north_xy = U16Vec2 {
        x: location.x,
        y: location.y.saturating_sub(1),
    };
    let east_xy = U16Vec2 {
        x: location.x + 1,
        y: location.y,
    };
    let north_layer = total_patches.get(&north_xy);
    let east_layer = total_patches.get(&east_xy);

    let top_corner_xy = U16Vec2 {
        x: location.x + 1,
        y: location.y.saturating_sub(1),
    };
    let top_corner = total_patches.get(&top_corner_xy);

    if north_layer.is_some() && east_layer.is_some() && top_corner.is_some() {
        if let Ok(path) = generate_land_mesh(
            total_patches.get(&location).unwrap(),
            north_layer.unwrap(),
            east_layer.unwrap(),
            top_corner.unwrap(),
        ) {
            completed_updates.push(LayerUpdate {
                path,
                position: total_patches
                    .get(&location)
                    .unwrap()
                    .terrain_header
                    .location,
            });
            patch_queue.remove(&location);
        }
    } else {
        patch_queue.insert(location, land);
    };
    completed_updates
}

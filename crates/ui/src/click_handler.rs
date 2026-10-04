use bevy::{
    app::{App, Plugin, Update},
    ecs::{hierarchy::ChildOf, message::MessageReader, system::Query},
    picking::{
        events::{Click, Pointer},
        mesh_picking::MeshPickingPlugin,
    },
};

use crate::mesh::ObjectData;

pub struct ClickHandlerPlugin;

impl Plugin for ClickHandlerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, handle_clicks)
            .add_plugins(MeshPickingPlugin);
    }
}

fn handle_clicks(
    mut events: MessageReader<Pointer<Click>>,
    objects: Query<&ObjectData>,
    parents: Query<&ChildOf>,
) {
    for event in events.read() {
        let mut entity = event.entity;

        loop {
            if let Ok(object) = objects.get(entity) {
                println!(
                    "Clicked object: {}, {}, {}",
                    object.id, object.scale, object.position
                );
                break;
            }

            let Ok(parent) = parents.get(entity) else {
                break;
            };

            entity = parent.parent();
        }
    }
}

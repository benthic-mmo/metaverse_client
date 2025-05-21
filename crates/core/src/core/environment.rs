use std::collections::HashMap;
use actix::{AsyncContext, Handler, Message};
use glam::U16Vec2;
use metaverse_environment::{land::Land, layer_handler::{parse_layer_data, PatchData, PatchLayer}};
use metaverse_messages::{environment::layer_data::LayerData, ui::ui_events::UiEventTypes};

use super::session::{Mailbox, UiMessage};

#[cfg(feature = "environment")]
#[derive(Debug, Message)]
#[rtype(result = "()")]
/// Contains the patch queue and patch cache.
pub struct EnvironmentCache {
    /// contains unprocessed patches that are yet to have their dependencies met.
    /// The dependencies are the required patches that live on their three corners.
    /// if the north, east and diagonal patches have not loaded in yet, they will remain in
    /// the patch queue until they come in.
    pub patch_queue: HashMap<U16Vec2, Land>,
    /// All of the patches that been received this session.
    pub patch_cache: HashMap<U16Vec2, Land>,
}

#[cfg(feature = "environment")]
impl Handler<LayerData> for Mailbox {
    type Result = ();
    fn handle(&mut self, msg: LayerData, ctx: &mut Self::Context) -> Self::Result {
        if let Some(session) = self.session.as_mut() {
            if let Ok(patch_data) = parse_layer_data(&msg) {
                match patch_data {
                    PatchLayer::Land(patches) => {
                        for land in patches {
                            session
                                .environment_cache
                                .patch_cache
                                .insert(land.terrain_header.location, land.clone());
                            let mut layer_updates = land.generate_ui_event(
                                &mut session.environment_cache.patch_queue,
                                &session.environment_cache.patch_cache,
                            );
                            let queue_save = session.environment_cache.patch_queue.clone();
                            for (_location, land) in queue_save {
                                layer_updates.extend(land.generate_ui_event(
                                    &mut session.environment_cache.patch_queue,
                                    &session.environment_cache.patch_cache,
                                ));
                            }

                            for layer in layer_updates {
                                ctx.address().do_send(UiMessage::new(
                                    UiEventTypes::LayerUpdateEvent,
                                    layer.to_bytes(),
                                ));
                            }
                        }
                    }
                    PatchLayer::Wind(_patches) => {}
                    PatchLayer::Water(_patches) => {}
                    PatchLayer::Cloud(_patches) => {}
                }
            }
        }
    }
}

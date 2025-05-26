use actix::{AsyncContext, Handler, Message};
use glam::U16Vec2;
use metaverse_environment::{
    land::Land,
    layer_handler::{PatchData, PatchLayer, parse_layer_data},
};
use metaverse_messages::{
    environment::layer_data::LayerData,
    ui::{
        mesh_update::{MeshType, MeshUpdate},
        ui_events::UiEventTypes,
    },
};
use std::collections::HashMap;

use crate::initialize::create_sub_share_dir;

use super::{
    generate_gltf::generate_high_lod,
    session::{Mailbox, UiMessage},
};

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
                            let mut layer_meshes = Vec::new();
                            if let Some(mesh) = land.clone().generate_mesh(
                                &mut session.environment_cache.patch_queue,
                                &session.environment_cache.patch_cache,
                            ) {
                                layer_meshes.push(mesh);
                            }
                            let queue_save = session.environment_cache.patch_queue.clone();
                            for (_location, land) in queue_save {
                                if let Some(mesh) = land.generate_mesh(
                                    &mut session.environment_cache.patch_queue,
                                    &session.environment_cache.patch_cache,
                                ) {
                                    layer_meshes.push(mesh);
                                }
                            }
                            for mesh in layer_meshes {
                                // generate gltf file from mesh
                                // create a UI event
                                if let Ok(dir) = create_sub_share_dir("land") {
                                    if let Ok(path) = generate_high_lod(
                                        &mesh,
                                        dir,
                                        land.terrain_header.filename.clone(),
                                    ) {
                                        ctx.address().do_send(UiMessage::new(
                                            UiEventTypes::MeshUpdate,
                                            MeshUpdate {
                                                position: mesh.position.unwrap(),
                                                path,
                                                mesh_type: MeshType::Land,
                                                id: None,
                                            }
                                            .to_bytes(),
                                        ));
                                    };
                                }
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

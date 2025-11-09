use super::session::Mailbox;
use crate::initialize::create_sub_share_dir;
use actix::{AsyncContext, Handler, Message};
use glam::U16Vec2;
use glam::Vec3;
use metaverse_environment::{
    land::Land,
    layer_handler::{parse_layer_data, PatchLayer},
};
use metaverse_messages::packet::message::UIMessage;
#[cfg(feature = "environment")]
use metaverse_messages::udp::environment::layer_data::LayerData;
use serde::Serialize;
use std::collections::HashMap;
use std::fs::File;
use std::io;
use std::io::Write;
use std::path::PathBuf;

use log::error;

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
        if let Some(session) = self.session.as_mut()
            && let Ok(patch_data) = parse_layer_data(&msg)
        {
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
                        for (mesh, coordinate) in layer_meshes {
                            use metaverse_messages::ui::land_update::{LandData, LandUpdate};
                            let scale = land.terrain_header.patch_size as f32;
                            let json_path = write_json(
                                &LandData {
                                    vertices: mesh.vertices,
                                    indices: mesh.indices,
                                    position: Vec3 {
                                        x: ((coordinate.x as f32) * scale),
                                        y: 0.0,
                                        z: ((coordinate.y as f32) * scale),
                                    },
                                },
                                land.terrain_header.filename.clone(),
                            )
                            .unwrap();
                            ctx.address()
                                .do_send(UIMessage::new_land_update(LandUpdate {
                                    path: json_path,
                                }));

                            //let path =
                            //    dir.join(format!("{}_high.glb", land.terrain_header.filename));

                            //if let Ok(_) = build_mesh_gltf(mesh, path.clone()) {
                            //    ctx.address()
                            //        .do_send(UIMessage::new_mesh_update(MeshUpdate {
                            //            // TODO: This is hardcoded to 16 because the patch
                            //            // sizes are all hardcoded to 16 right now. This should
                            //            // be fixed when that bug is resolved.
                            //            position: Vec3 {
                            //                x: (coordinate.x as f32) * 16.0,
                            //                y: (coordinate.y as f32) * 16.0,
                            //                z: 0.0,
                            //            },
                            //            path,
                            //            mesh_type: MeshType::Land,
                            //            id: None,
                            //        }));
                            //}
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

/// When an object is retrieved in full, the data will be written in serializable json format, to
/// create a cache. The JSON will then be sent to another crate to convert it into a 3d model that
/// can be rendered.
fn write_json<T: Serialize>(data: &T, filename: String) -> io::Result<PathBuf> {
    match create_sub_share_dir("land") {
        Ok(mut agent_dir) => match serde_json::to_string(&data) {
            Ok(json) => {
                agent_dir.push(format!("{}.json", filename));
                let mut file = File::create(&agent_dir).unwrap();
                file.write_all(json.as_bytes()).unwrap();
                Ok(agent_dir)
            }
            Err(e) => {
                error!("Failed to serialize scene group {:?}, {:?}", filename, e);
                Err(io::Error::other(e))
            }
        },
        Err(e) => {
            error!(
                "Failed to create agent dir for {:?}. Unable to cache downloaded items.",
                e
            );
            Err(io::Error::other(e))
        }
    }
}

use super::session::Mailbox;
use crate::initialize::create_sub_share_dir;
use crate::session::SendUIMessage;
use actix::{AsyncContext, Handler, Message};
use glam::U16Vec2;
use glam::Vec3;
use log::error;
use log::warn;
use metaverse_environment::{
    land::Land,
    layer_handler::{parse_layer_data, PatchLayer},
};
use metaverse_messages::packet::message::UIMessage;
use metaverse_messages::udp::environment::layer_data::LayerData;
use metaverse_messages::ui::land_update::{LandData, LandUpdate};
use serde::Serialize;
use std::collections::HashMap;
use std::fs::File;
use std::io;
use std::io::Write;
use std::path::PathBuf;

/// Contains the patch queue and patch cache.
#[derive(Debug)]
pub struct EnvironmentCache {
    /// contains unprocessed patches that are yet to have their dependencies met.
    /// The dependencies are the required patches that live on their three corners.
    /// if the north, east and diagonal patches have not loaded in yet, they will remain in
    /// the patch queue until they come in.
    pub patch_queue: HashMap<U16Vec2, Land>,
    /// All of the patches that been received this session.
    pub patch_cache: HashMap<U16Vec2, Land>,
}

/// Message to handle new layer data coming in from the server
///
/// Handles land, water, cloud and wind patches. This decoding and handling is done in the
/// metaverse-environment crate. This is mainly used for maintaining the session's knowledge of land
/// tiles.  
///
/// # Cause
/// - LayerData packet received from UDP socket
///
/// # Effect
/// - Dispatches a [`LandUpdate`] packet to the UI
#[derive(Message)]
#[rtype(result = "()")]
pub struct HandleLayerData {
    /// The layer data packet to process
    pub layer_data: LayerData,
}

#[cfg(feature = "environment")]
impl Handler<HandleLayerData> for Mailbox {
    type Result = ();
    fn handle(&mut self, msg: HandleLayerData, ctx: &mut Self::Context) -> Self::Result {
        if let Some(session) = self.session.as_mut()
            && let Ok(patch_data) = parse_layer_data(&msg.layer_data)
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
                                .do_send(SendUIMessage{ui_message: UIMessage::new_land_update(LandUpdate {
                                    path: json_path,
                                })});
                        }
                    }
                }
                PatchLayer::Wind(_patches) => {
                    // TODO: implement wind patch 
                    warn!("Wind patch received. Currently unimplemented.");
                }
                PatchLayer::Water(_patches) => {
                    // TODO: implement water patch 
                    warn!("Water patch received. Currently unimplemented.");
                }
                PatchLayer::Cloud(_patches) => {
                    // TODO: implement cloud patch
                    warn!("Cloud patch received. Currently unimplemented.");
                }
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

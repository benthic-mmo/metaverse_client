use std::collections::HashMap;

use bitreader::{BitReader, BitReaderError};
use bytemuck::cast_slice;
use glam::U16Vec2;
use log::warn;
use metaverse_messages::{
    environment::layer_data::LayerData, ui::layer_update::LayerUpdate,
};
use twox_hash::XxHash32;

use crate::{
    error::PatchError,
    generate_mesh::generate_land_mesh,
    layer_handler::{
        END_OF_PATCHES, PatchData, TerrainHeader, bits_to_big_endian, decompress_patch, read_bits,
    },
};

// This handles receiving and parsing LayerData packets.
// The LayerData packet system is very poorly documented.

/// This is the default size of the patch
const PATCHES_PER_EDGE: u16 = 16;
/// The land struct. Can contain both LandExtended and Land.
#[derive(Debug, Clone)]
pub struct Land {
    /// The terrain header, the struct that contains information useful for decoding and
    /// decompression
    pub terrain_header: TerrainHeader,
    /// The generated heightmap. This contains the array of decoded height values.
    pub heightmap: Vec<f32>,
}

/// creates a Land object from a LayerData packet
impl PatchData for Land {
    fn from_packet(packet: &LayerData, extended: bool) -> Result<Vec<Self>, PatchError> {
        let mut patches = Vec::new();
        let mut reader = BitReader::new(&packet.layer_content);
        // each Layerdata packet can contain several patches. This loops through each sub-patch.
        loop {
            // create the header from bytes
            let mut terrain_header = match TerrainHeader::from_bytes(&mut reader, extended) {
                Ok(mut header) => {
                    header.stride = packet.stride;
                    header.patch_size = packet.patch_size;
                    header
                }
                Err(e) => {
                    return Err(PatchError {
                        message: format!("Failed to create header: {}", e).to_string(),
                    });
                }
            };

            // quick check to see if the data is valid
            if !extended
                && (terrain_header.location.x >= PATCHES_PER_EDGE
                    || terrain_header.location.y >= PATCHES_PER_EDGE)
            {
                warn!("Invalid LayerData packet {:?}", terrain_header);
            }

            // If the quantized_world_bits is 97, that is the signal to quit parsing.
            if terrain_header.quantized_world_bits == END_OF_PATCHES {
                break;
            }

            // read the heightmap bits
            let patch = parse_heightmap(&mut reader, &terrain_header)?;
            // this hashes the read bits. Patches that have the same geometry will have the same
            // hash, allowing you to easily check if there has been an update to the terrain.
            // this will be useful for caching and clearing the cache later.
            let hash = XxHash32::oneshot(1234, cast_slice(&patch));
            terrain_header.filename = format!(
                "{}_{}_{}",
                &terrain_header.location.x, &terrain_header.location.y, hash
            );

            // this decompresses the data using JPEG type decompression
            let heightmap = decompress_patch(&terrain_header, &patch);

            patches.push(Land {
                terrain_header,
                heightmap,
            });
        }
        Ok(patches)
    }

    /// This handles generating the geometry for the land packet, and sending the UI signals to load
    /// the patch geometry.
    /// Accepts the land data to render. The patch queue is the queue that contains all patches that
    /// failed to be generated on the first pass. Total patches contains all of the patches that have been
    /// read to this point.
    ///
    /// This should be called in a loop against the output of Layer::parse_packet.
    /// Currently this is called from within the actix mailbox. It receives the layer
    fn generate_ui_event(
        self: Self,
        retry_queue: &mut HashMap<U16Vec2, Self>,
        total_patches: &HashMap<U16Vec2, Self>,
    ) -> Vec<LayerUpdate> {
        let location = self.terrain_header.location;
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
                retry_queue.remove(&location);
            }
        } else {
            retry_queue.insert(location, self);
        };
        completed_updates
    }
}

/// parse_heightmap takes the bitreader, and the terrain header.
/// this runs an algorithm to retrieve the raw data from the packet, which will be later
/// decompressed.
pub fn parse_heightmap(
    reader: &mut BitReader,
    terrain_header: &TerrainHeader,
) -> Result<Vec<f32>, BitReaderError> {
    let patch_size = terrain_header.patch_size as usize;
    let total = patch_size * patch_size;
    let mut patch: Vec<f32> = vec![0.0; total];

    for i in 0..total {
        if reader.read_bool()? {
            if reader.read_bool()? {
                let is_negative = reader.read_bool()?;
                let bits = read_bits(reader, terrain_header.world_bits)?;
                let magnitude = bits_to_big_endian(&bits, 8) as f32;
                patch[i] = if is_negative { -magnitude } else { magnitude };
            } else {
                for j in patch.iter_mut().take(total).skip(i) {
                    *j = 0.0;
                }
                break;
            }
        } else {
            patch[i] = 0.0;
            continue;
        }
    }
    Ok(patch)
}

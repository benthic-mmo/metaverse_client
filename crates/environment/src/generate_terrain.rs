use bitreader::{BitReader, BitReaderError};
use bytemuck::cast_slice;
use log::{error, warn};
use twox_hash::XxHash32;

use crate::{
    constants::{build_copy_matrix16, build_dequantize_table16, idct_column16, idct_line16},
    generate_mesh::generate_land_mesh,
};
use glam::{U16Vec2, u16, u32, usize};
use metaverse_messages::{layer_data::{LayerData, LayerType}, ui::custom::layer_update::LayerUpdate};

// This handles receiving and parsing LayerData packets.
// The LayerData packet system is very poorly documented.
//
/// When these bytes are read as the quantized_world_bits, it signifies the end of the patches
/// contained within the LayerData packet.
const END_OF_PATCHES: u8 = 97;
/// This is the default size of the patch
const PATCHES_PER_EDGE: u16 = 16;
/// this is the copy matrix, used for decoding the encoded patch data.
static COPY_MATRIX_16: [usize; 256] = build_copy_matrix16();
/// This is the dequantize table that is used to multiply the compressed data by a factor it was
/// divided by, returning it to a f32 after compression.
static DEQUANTIZE_TABLE_16: [f32; 256] = build_dequantize_table16();

/// This parses the incoming LayerData packet into its proper type.
pub fn parse_layer_data(data: &LayerData) -> Option<Vec<LayerUpdate>> {
    match data.layer_type {
        LayerType::Land => match Land::from_packet(data, false) {
            Ok(layer_info) => Some(layer_info),
            Err(e) => {
                error!("Error parsing Land packet: {}", e);
                None
            }
        },
        LayerType::LandExtended => match Land::from_packet(data, true) {
            Ok(layer_info) => Some(layer_info),
            Err(e) => {
                error!("Error parsing LandExtended packet: {}", e);
                None
            }
        },
        LayerType::Wind | LayerType::WindExtended => {
            println!("wind");
            None
        }
        LayerType::Water | LayerType::WaterExtended => {
            println!("water");
            None
        }
        LayerType::Cloud | LayerType::CloudExtended => {
            println!("cloud");
            None
        }
        LayerType::Unknown => {
            println!("unknown");
            None
        }
    }
}

/// This is the header that begins each new terrain patch, containing information required for
/// parsing.
#[derive(Debug, Clone)]
pub struct TerrainHeader {
    /// This is used to determine if we have reached the end of patches, along with determining the
    /// world_bits.
    pub quantized_world_bits: u8,
    /// this scales the decompressed data back to a real-world value  
    pub dc_offset: f32,
    /// a multiplioer used for decompression
    pub range: u16,
    /// A boolean to determine if it is an extended region or not
    pub extended_region: bool,
    /// determines how many bits to use when representing a height before decompression
    pub world_bits: u32,
    /// the xy location of where this patch is on the grid
    pub location: U16Vec2,
    // fields from the layerData packet
    /// stride is the length of the data for each sub-packet
    pub stride: u16,
    /// The size of the patch. Should always be 16.
    pub patch_size: u8,
    /// this is the filename of the terrain object, for what will be rendered by the UI.
    /// this is formatted x_y_<hash>
    pub filename: String,
}
impl TerrainHeader {
    /// using a BitReader, deconstruct the terrainHeader from the bytes in the reader
    /// takes extended_region as a bool
    pub fn from_bytes(
        reader: &mut BitReader,
        extended_region: bool,
    ) -> Result<Self, BitReaderError> {
        let quantized_world_bits = reader.read_u8(8)?;
        if quantized_world_bits == END_OF_PATCHES {
            return Ok(TerrainHeader {
                quantized_world_bits,
                dc_offset: 0.0,
                range: 0,
                extended_region: false,
                world_bits: 0,
                location: U16Vec2 { x: 0, y: 0 },
                patch_size: 0,
                stride: 0,
                filename: "".to_string(),
            });
        }

        let dc_offset_bits = reader.read_u32(32)?.swap_bytes();
        let dc_offset = f32::from_bits(dc_offset_bits);

        let range = reader.read_u16(16)?.swap_bytes();

        // The patch_ids is a very weird peice of code.
        // this is the xy coordinate of  where the patch is located on the grid.
        // if it is a large region, the xy coordinate will be 32 bits, but if it isn't, it will be 10
        // bits. To handle the x and y, it reads the first half as x, and the second half as y.
        // so for a regular patch, it will be 5 bits of x, followed by 5 bits of y.
        let patch_ids = if extended_region {
            read_bits(reader, 32)?
        } else {
            read_bits(reader, 10)?
        };

        // The bits are laid out in a way that they need to be parsed as 8 bits, with the following
        // bits appended to the front.
        let patch_decoded = bits_to_big_endian(&patch_ids, 8);

        // This uses only the first 16 or first 5 bits to set the x position.
        let x = if extended_region {
            patch_decoded.checked_shr(16).unwrap_or(0)
        } else {
            patch_decoded.checked_shr(5).unwrap_or(0)
        };

        // this uses only the last 16 or last 5 bits to set the y position.
        let y = if extended_region {
            patch_decoded & 0xFFFF
        } else {
            patch_decoded & 0x1F
        };

        // TODO: figure out what is up with this 0x0f
        let world_bits = ((quantized_world_bits & 0x0f) + 2) as u32;
        Ok(TerrainHeader {
            quantized_world_bits,
            dc_offset,
            range,
            extended_region,
            world_bits,
            // these can safely be cast to u16 because they only contain half of a u32.
            location: U16Vec2 {
                x: x as u16,
                y: y as u16,
            },
            patch_size: 0,
            stride: 0,
            filename: "".to_string(),
        })
    }
}

/// The land struct. Can contain both LandExtended and Land.
#[derive(Debug, Clone)]
pub struct Land {
    pub terrain_header: TerrainHeader,
    pub heightmap: Vec<f32>,
}
impl Land {
    /// creates a Land object from a LayerData packet
    pub fn from_packet(data: &LayerData, extended: bool) -> Result<Vec<LayerUpdate>, BitReaderError> {
        let mut patches = Vec::new();
        let mut reader = BitReader::new(&data.layer_content);
        // each Layerdata packet can contain several patches. This loops through each sub-patch.
        loop {
            // create the header from bytes
            let mut terrain_header = match TerrainHeader::from_bytes(&mut reader, extended) {
                Ok(mut header) => {
                    header.stride = data.stride;
                    header.patch_size = data.patch_size;
                    header
                }
                Err(e) => return Err(e),
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

            match generate_land_mesh(&terrain_header, heightmap) {
                Ok(path) => {
                    patches.push(LayerUpdate {
                        path,
                        position: terrain_header.location.clone(),
                    });
                }
                Err(e) => {
                    warn!("Failed to generate land mesh: {:?}", e);
                }
            };
        }
        Ok(patches)
    }
}

fn parse_heightmap(
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

/// Decompresss the heightmap data using JPEG like decompression
pub fn decompress_patch(terrain_header: &TerrainHeader, patch: &[f32]) -> Vec<f32> {
    let patch_size = terrain_header.patch_size as usize;
    let mut block: Vec<f32> = vec![0.0; patch_size * patch_size];
    let mut output: Vec<f32> = vec![0.0; patch_size * patch_size];
    let prequant = terrain_header.quantized_world_bits >> (4 + 2);
    let prequant = if prequant > 0 { prequant } else { 1 };
    let quantize = 1 << prequant;
    let ooq = 1.0f32 / quantize as f32;
    let mult = ooq * terrain_header.range as f32;
    let addval = mult * (1 << (prequant - 1)) as f32 + terrain_header.dc_offset;

    if terrain_header.patch_size == 16 {
        for i in 0..block.len() - 1 {
            block[i] = patch[COPY_MATRIX_16[i]] * DEQUANTIZE_TABLE_16[i];
        }
        let mut temp: Vec<f32> = vec![0.0; 16 * 16];
        for i in 0..16 {
            idct_column16(&block, &mut temp, i);
        }
        for i in 0..16 {
            idct_line16(&temp, &mut block, i);
        }
    } else {
        println!("patch size unsupported")
    }
    for i in 0..block.len() - 1 {
        output[i] = block[i] * mult + addval;
    }
    output
}

/// This handles the big-endianness of the bit parsing
/// you might have raw data that looks like this after removing the headers from the packet
/// 0 0 0 164 65 1 0 8 96.....
/// | |_______|  |_| |__| [8, 96] are the bytes containing x and y.
/// |         |    |    | Read as u8s, the values are
/// |         |    |    | 8          96
/// |         |    |    | 00001000   01100000
/// |         |    |    | To handle 10 bytes, you read the whole of the first value, and the first
/// |         |    |    | two bits of the second.
/// |         |    |    | 00001000  01
/// |         |    |    | Because it is big-endian, you have to move the two bits to the beginning
/// |         |    |    | of the data and then parse as a u32.
/// |         |    |    | 0100001000
/// |         |    |    | When getting x and y using masks this comes out to:
/// |         |    |    | 01000 01000
/// |         |    |    | x: 8, y: 8
/// |         |    |    | This is the same case for 32 bytes, though a little easier to see.
/// |         |    |
/// |         |    |[1,0] u16 value of the range
/// |         | [0, 0, 164, 65] f32 value of the dc_offset_bits
/// | u8 value of the quantized_world_bits
///
pub fn bits_to_big_endian(bits: &[u32], chunk_size: usize) -> u32 {
    let mut value: u32 = 0;
    for chunk in bits.chunks(chunk_size).rev() {
        for &bit in chunk {
            value = (value << 1) | bit
        }
    }
    value
}

// a simple bit reader that returns a vec of u32s of the bits.
pub fn read_bits(reader: &mut BitReader, bit_count: u32) -> Result<Vec<u32>, BitReaderError> {
    let mut bits = Vec::with_capacity(bit_count as usize);
    for _ in 0..bit_count {
        let bit = reader.read_bool()?; // reads 1 bit
        bits.push(if bit { 1 } else { 0 });
    }
    Ok(bits)
}

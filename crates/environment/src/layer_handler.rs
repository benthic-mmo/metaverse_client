use bitreader::{BitReader, BitReaderError};
use metaverse_messages::{
    udp::environment::layer_data::{LayerData, LayerType},
    utils::render_data::RenderObject,
};
use std::collections::HashMap;

use crate::{
    cloud::Cloud,
    constants::{build_copy_matrix16, build_dequantize_table16, idct_column16, idct_line16},
    error::PatchError,
    land::Land,
    water::Water,
    wind::Wind,
};
use glam::{U16Vec2, u16, u32, usize};

/// this is the copy matrix, used for decoding the encoded patch data.
static COPY_MATRIX_16: [usize; 256] = build_copy_matrix16();
/// This is the dequantize table that is used to multiply the compressed data by a factor it was
/// divided by, returning it to a f32 after compression.
static DEQUANTIZE_TABLE_16: [f32; 256] = build_dequantize_table16();
/// When these bytes are read as the quantized_world_bits, it signifies the end of the patches
/// contained within the LayerData packet.
pub const END_OF_PATCHES: u8 = 97;

/// This is the enum so I can use a generic type for the vector the parse layer data function
/// returns.
pub enum PatchLayer {
    /// a vector of lands
    Land(Vec<Land>),
    /// a vector of winds
    Wind(Vec<Wind>),
    /// a vector of waters
    Water(Vec<Water>),
    /// a vector of clouds
    Cloud(Vec<Cloud>),
}

/// Handles the LayerData packet after parsing its headers
pub fn parse_layer_data(data: &LayerData) -> Result<PatchLayer, PatchError> {
    match data.layer_type {
        LayerType::Land | LayerType::LandExtended => {
            let extended = matches!(data.layer_type, LayerType::LandExtended);
            let result = Land::from_packet(data, extended)?;
            Ok(PatchLayer::Land(result))
        }
        LayerType::Wind | LayerType::WindExtended => {
            let extended = matches!(data.layer_type, LayerType::WindExtended);
            let result = Wind::from_packet(data, extended)?;
            Ok(PatchLayer::Wind(result))
        }
        LayerType::Water | LayerType::WaterExtended => {
            let extended = matches!(data.layer_type, LayerType::WaterExtended);
            let result = Water::from_packet(data, extended)?;
            Ok(PatchLayer::Water(result))
        }
        LayerType::Cloud | LayerType::CloudExtended | LayerType::Unknown => {
            let extended = !matches!(data.layer_type, LayerType::Cloud);
            let result = Cloud::from_packet(data, extended)?;
            Ok(PatchLayer::Cloud(result))
        }
    }
}

/// The PatchData trait, used for all of the patch data.
pub trait PatchData: Sized {
    /// from_packet converts the LayerData packet into a vector of self
    fn from_packet(packet: &LayerData, extended: bool) -> Result<Vec<Self>, PatchError>;

    /// generate_ui_event generates a vector of LayerUpdate for the UI to handle
    fn generate_mesh(
        self,
        retry_queue: &mut HashMap<U16Vec2, Self>,
        total_patches: &HashMap<U16Vec2, Self>,
    ) -> Option<(RenderObject, U16Vec2)>;
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
    /// stride is the length of the data for each patch
    pub stride: u16,
    /// The size of the patch. Should always be 16.
    pub patch_size: u8,
    /// this is the filename of the terrain object, for what will be rendered by the UI.
    /// this is formatted `x_y_<hash>`
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

/// Decompresss the heightmap data using JPEG like decompression
pub fn decompress_patch(terrain_header: &TerrainHeader, patch: &[f32]) -> Vec<f32> {
    let patch_size = terrain_header.patch_size as usize;
    let mut block: Vec<f32> = vec![0.0; patch_size * patch_size];
    let mut output: Vec<f32> = vec![0.0; patch_size * patch_size];
    let prequant = (terrain_header.quantized_world_bits >> 4) + 2;
    let prequant = if prequant > 0 { prequant } else { 1 };
    let quantize = 1 << prequant;
    let ooq = 1.0f32 / quantize as f32;
    let mult = ooq * terrain_header.range as f32;
    let addval = mult * (1 << (prequant - 1)) as f32 + terrain_header.dc_offset;
    if terrain_header.patch_size == 16 {
        for i in 0..block.len() {
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
    for i in 0..block.len() {
        output[i] = block[i] * mult + addval;
    }
    output
}

/// This handles the big-endianness of the bit parsing
/// see [Explanation](crate#bit-packing-information) for why this needs to be done.
pub fn bits_to_big_endian(bits: &[u32], chunk_size: usize) -> u32 {
    let mut value: u32 = 0;
    for chunk in bits.chunks(chunk_size).rev() {
        for &bit in chunk {
            value = (value << 1) | bit
        }
    }
    value
}

/// a simple bit reader that returns a vec of u32s of the bits.
/// BitReader won't work for this, due to the weird way the binary is being handled.
pub fn read_bits(reader: &mut BitReader, bit_count: u32) -> Result<Vec<u32>, BitReaderError> {
    let mut bits = Vec::with_capacity(bit_count as usize);
    for _ in 0..bit_count {
        let bit = reader.read_bool()?; // reads 1 bit
        bits.push(if bit { 1 } else { 0 });
    }
    Ok(bits)
}

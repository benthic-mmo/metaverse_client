//! # Metaverse Environment
//! Environment library for open metaverse clients!
//!
//! Provides handling for environment data sent from an open metaverse server.
//! Will handle compression and decompression for heightmap files, and handling LayerData packets
//! sent from the server.
//!
//!
//! # Packet spec
//! The LayerData Packet is laid out like this
//!
//! | LayerData |          |                      |                   |                     |                   |
//! |-------|--------------|----------------------|-------------------|---------------------|-------------------|
//! | Packet Header| id:11        | reliable: true       | zerocoded: false  |      frequency: low |                   |
//!
//! ## Body Header
//! |BodyHeader |         |       |                                                     |
//! |-----------|---------|-------|-----------------------------------------------------|
//! | Type      | 1 byte  | [u8]  | The type of the patch. Land, Water, Wind and Cloud. |
//! | Length    | 4 bytes | [u16] | The length of the packet's patch data. For initializing a buffer.|
//! | Stride    | 1 byte  | [u8]  | The length of the data for each patch.              |
//! | Patch Size| 1 byte  | [u8]  | The size of the patches. Should always be 16.       |
//! | Type      | 1 byte  | [u8]  | A redundant type value.                             |
//! | Content   | variable bytes (read to end) | PatchData | Compressed patch data      |
//!
//! The body header is followed by a byte array the length of the header's length field, containing several individual
//! patches.
//! ## Patch Spec
//! | Patch data|                   |                                                                      |
//! |-----------|-------------------|----------------------------------------------------------------------|
//! | Quantized World Bits | 1 byte | [u8] | Used for checking the end of patches, and calulating read size|
//! | DC Offset | 4 bytes  | [f32]  | Used to scale the decompressed data back to a real world value       |
//! | Range     | 2 bytes  | [u16]  | A multiplier used for decompression                                  |
//! | Patch IDs | 10 or 4 bytes |   | A compressed way to store the xy location of the patch    .          |
//! | Compressed Layer Data| vriable bytes (read to end)| | compressed data                                |
//!
//! For Patch IDs, If the region is extended, the first 5 bits of the 10 bit string are used for the x, and the next 5
//! represent the y. If the region is not extended, the first 2 bytes represent the x, and the next 2 represent
//! the y. Stored in big-endian format.
//!
//!
//! # Patch Spec
//! # Bit packing information
//! Throughout this crate, occasionally bit values in the data are packed in a way that does not align them to
//! a byte. This can be confusing, and cause for frustrating debugging.
//!
//! For example:
//!
//! Patch ids is used for determining the xy position of the patch.
//! This can be either 10 bits, or 32 bits, depending on if it is an extended patch or not.
//! The binary is read into a buffer, with the first half of the bytes being used as the x value,
//! and the second half of the bytes being used as the y value.
//!
//! However, the binary is laid out in big-endian format, which makes this very difficult to see.
//! you might have raw data that looks like this after removing the headers from the packet
//! ```text
//! Bytes:     0    0    0   164  65   1    0    8   96 ...
//!            |    |    |    |   |    |    |    |    |
//!            |    |    |    |   |    |    |    └────└─ [8, 96] patch ID bytes
//!            |    |    |    |   |    └────└─ [1,0] range (2 bytes)
//!            |    └────└────└───└─[0, 0, 164, 65] f32 DC offset (4 bytes)
//!            └─[0] quantized_world_bits (1 byte)
//!```
//!
//! Read as u8s, the values are
//!
//! |8       | 96     |
//! |--------|--------|
//! |00001000|01100000|
//!
//! To handle 10 bytes, you read the whole of the first value, and the first
//! two bits of the second.
//!
//! |00001000 |  01 |
//! |---------|-----|
//!
//! Because it is big-endian, you have to move the two bits to the beginning
//! of the data and then parse as a u32.
//!
//! |01 | 00001000 |
//! |---|----------|
//!
//! The resulting XY values come out to
//!
//! | 01000 | 01000 |
//! |-------|-------|
//! |x: 8   | y: 8  |
//!
//! <div class="warning"> This crate is in development! </div>
//! This crate is not ready for public use! Most method are missing, and it is very unpolished.

#![warn(missing_docs)]
/// This module contains constants used for decompression
pub mod constants;
/// error definitions
pub mod error;
/// This module handles generating the triangles required for generating the GLTF files.
pub mod generate_triangles;
/// This module handles parsing and decoding LayerData packets
pub mod layer_handler;

/// Contains information and handling for land patches
pub mod land;

/// TODO: unimplemented
pub mod cloud;
/// TODO: unimplemented
pub mod water;
/// TODO: unimplemented
pub mod wind;

/// Generate the GLTF files for patch data
pub mod generate_gltf;

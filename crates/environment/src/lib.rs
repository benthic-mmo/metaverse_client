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
//! | Body Header  | type: 8 bytes| [^1][^2]length: 16 bytes | stride: 8 bytes   | patch size: 8 bytes | [^1]type: 8 bytes |
//!
//! The body header is followed by a byte array the length of the header's length field, containing several individual
//! patches.
//!
//! # Patch Spec
//! |Patch data|                       |                       |                |                              |
//! |-------|--------------------------|-----------------------|----------------|------------------------------|
//! |Patch Header | quant world bits: 8 bits | dc offset: 32 bits    | range: 16 bits | patch ids: 10 or 32 bits     |
//! |Patch | Compressed layer data    |                       |                |                              |
//!
//! [^1]: These fields are unused by this implementation, but still need to be accounted for
//! when deserializing
//!
//! [^2]: This represents the size of the patch data following the header. This is useful for
//! initializing the buffer that the compressed patches will be read into.
//!
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
/// This module handles generating the gltf files that will be rendered by the UI.
pub mod generate_mesh;
/// This module handles parsing and decoding LayerData packets
pub mod layer_handler;

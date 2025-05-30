/// # Layer Data
/// <https://wiki.secondlife.com/wiki/LayerData>
///
/// The layer data packet. Contains a list of patches that will be decompressed into patches of
/// land, water, cloud and wind.
///
/// ## Header
/// | LayerData     |              |                      |                   |                      |
/// |---------------|--------------|----------------------|-------------------|----------------------|
/// | Packet Header | id:11        | reliable: true       | zerocoded: false  |      frequency: low  |
///
/// ## Body Header
/// This is the header for the body. This follows the packet header, and only appears once. It
/// gives information about how to handle the list of patches contained within the packet.
/// |BodyHeader |         |       |                                                     |
/// |-----------|---------|-------|-----------------------------------------------------|
/// | Type      | 1 byte  | [u8]  | The type of the patch. Land, Water, Wind and Cloud. |
/// | Length [^1]| 2 bytes | [u16] | The length of the packet's patch data.              |
/// | Stride    | 1 byte  | [u8]  | The length of the data for each patch.              |
/// | Patch Size| 1 byte  | [u8]  | The size of the patches. Should always be 16.       |
/// | Type[^1]  | 1 byte  | [u8]  | A redundant type value.                             |
/// | Content   | variable bytes (read to end) | PatchData | Compressed patch data      |

/// ## Patch Spec
/// This is the header structure for each of the internal patches.
/// | Patch data|                   |                                                                      |                                                                    |
/// |-----------|----------|--------|----------------------------------------------------------------------|
/// | Quantized World Bits | 1 byte | [u8] | Used for checking the end of patches, and calulating read size|
/// | DC Offset | 4 bytes  | [f32]  | Used to scale the decompressed data back to a real world value       |
/// | Range     | 2 bytes  | [u16]  | A multiplier used for decompression                                  |
/// | Patch IDs | 10 or 4 bytes |   | A compressed way to store the xy location of the patch[^1].          |
/// | Compressed Layer Data| vriable bytes (read to end)| | compressed data                                |
///
/// [^1]: These fields are unused by this implementation, but still need to be accounted for
/// when deserializing
///
/// [^2]: If the region is extended, the first 5 bits of the 10 bit string are used for the x, and the next 5
/// represent the y. If the region is not extended, the first 2 bytes represent the x, and the next 2 represent
/// the y. Stored in big-endian format.
pub mod layer_data;

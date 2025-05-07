/// # Header
///<https://wiki.secondlife.com/wiki/Packet_Layout>
///
/// Handles reading and writing headers for packets.
///
/// ## Header Structure
/// | Header ||||
/// |----------------|---------|-------|---------------------------------|
/// | Flags          | 1 byte         | | Zerocoded (0x80), reliable (0x40), resent(0x20) and ack(0x10) flags, set with bitwise and. Last 4 bits are unused. |
/// | Sequence Number| 4 bytes         | [u32] | sequence number of sent packets |
/// | Extra          | 1 byte          | [u8]  | How many extra bytes of header is there left to read |
/// | Frequency      | 1, 2 or 4 bytes | | The packet ID and the frequency of the packet |
/// | Appended Acks  | Variable bytes  | | the rest of the packet is filled with as many acks from previous messages as will fit.
///
pub mod header;

/// # Packet
/// <https://wiki.secondlife.com/wiki/Packet_Layout>
///
/// Contains information about the packet layout used for serializing and deserializing packets.
pub mod packet;

/// Contains structs and enums used for determining types of packets based on header values and
/// assigning those valeus to rust data types.
pub mod packet_types;

use super::header::Header;
use super::packet_types::PacketType;
use actix::prelude::*;
use std::any::Any;
use std::io;
use std::io::{Cursor, Read};

#[derive(Debug, Message, Clone)]
#[rtype(result = "()")]
/// The base type for all packets in the spec.
/// contains a header and a body.
pub struct Packet {
    /// The header of the packet. All valid packets implement this header with this layout.
    pub header: Header,
    /// Body of type PacketType, containing all of the different types of packets in the spec.
    pub body: PacketType,
}

/// this is the trait that allows for serializing and deserializing the packet's data
/// All packets need to be able to convert to and from bytes in order to be sent to and from the
/// server, and to and from the client to the UI.
pub trait PacketData: std::fmt::Debug + Send + Sync + 'static + Any {
    /// convert from bytes to the packet type
    fn from_bytes(bytes: &[u8]) -> io::Result<Self>
    where
        Self: Sized;
    /// convert to bytes from the packet type
    fn to_bytes(&self) -> Vec<u8>;
}

impl Packet {
    /// Read bytes and convert it to a packet.
    /// First parse the packet's header, and then parse the packet's body based on the ID parsed
    /// from the header.
    pub fn from_bytes(bytes: &[u8]) -> io::Result<Self> {
        let header = Header::try_from_bytes(bytes)?;
        // if the packet has a body, add the body to the packet
        let body = if header.size.unwrap_or(0) < bytes.len() {
            &bytes[header.size.unwrap_or(0)..]
        } else {
            &[]
        };
        let body_bytes = if header.zerocoded {
            zero_decode(body)
        } else {
            body.to_vec() // Convert slice to Vec<u8>
        };

        let body = match PacketType::from_id(header.id, header.frequency, body_bytes.as_slice()) {
            Ok(parsed_body) => parsed_body, // If parsing succeeds, use the parsed body
            Err(e) => {
                return Err(e);
            }
        };
        Ok(Self { header, body })
    }

    /// convert a packet to bytes for sending.
    /// simply call the header and body's to_bytes() functions
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend(self.header.to_bytes());
        bytes.extend(self.body.to_bytes());
        bytes
    }
}

/// decompress zero encoded packets for parsing
fn zero_decode(bytes: &[u8]) -> Vec<u8> {
    let mut cursor = Cursor::new(bytes);
    let mut dest = Vec::new();

    while cursor.position() < bytes.len() as u64 {
        let mut byte = [0u8; 1];
        cursor.read_exact(&mut byte).unwrap();
        let byte = byte[0];

        if byte == 0x00 {
            let mut repeat_count = [0u8; 1];
            cursor.read_exact(&mut repeat_count).unwrap();
            let repeat_count = repeat_count[0] as usize;

            dest.extend(vec![0x00; repeat_count]);
        } else {
            dest.push(byte);
        }
    }
    dest
}

// not implemented yet
fn _zero_encode(src: &[u8]) -> Vec<u8> {
    let mut dest = Vec::new();
    let mut i = 0;

    while i < src.len() {
        if src[i] == 0x00 {
            // Count consecutive zeros
            let mut count = 1;
            while i + count < src.len() && src[i + count] == 0x00 {
                count += 1;
            }

            // Add a zero byte and the repeat count to the destination
            dest.push(0x00);
            dest.push(count as u8);
            i += count;
        } else {
            // Add non-zero byte to the destination
            dest.push(src[i]);
            i += 1;
        }
    }

    dest
}

use super::header::Header;
use super::packet_types::PacketType;
use crate::errors::ParseError;
use actix::Message;
use byteorder::ReadBytesExt;
use std::any::Any;
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
    fn from_bytes(bytes: &[u8]) -> Result<Self, ParseError>
    where
        Self: Sized;
    /// convert to bytes from the packet type
    fn to_bytes(&self) -> Vec<u8>;
}

impl Packet {
    /// Read bytes and convert it to a packet.
    /// First parse the packet's header, and then parse the packet's body based on the ID parsed
    /// from the header.
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, ParseError> {
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

        let body = PacketType::from_id(header.id, header.frequency, body_bytes.as_slice())?;

        Ok(Self { header, body })
    }

    /// convert a packet to bytes for sending.
    /// simply call the header and body's to_bytes() functions. If it is zerocoded, don't zerocode
    /// the first six bytes of the header. for whatever reason.
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend(self.header.to_bytes());
        bytes.extend(self.body.to_bytes());
        if self.header.zerocoded {
            let mut zeroed = zero_encode(&bytes[6..]);
            let mut final_bytes = Vec::with_capacity(6 + zeroed.len());
            final_bytes.extend_from_slice(&bytes[0..6]);
            final_bytes.append(&mut zeroed);
            return final_bytes;
        }
        bytes
    }
}

/// decompress zero encoded packets for parsing
fn zero_decode(bytes: &[u8]) -> Vec<u8> {
    let mut cursor = Cursor::new(bytes);
    let mut dest = Vec::new();

    while let Ok(byte) = cursor.read_u8() {
        if byte == 0x00 {
            if let Ok(count) = cursor.read_u8() {
                dest.extend(std::iter::repeat(0x00).take(count as usize));
            } else {
                dest.push(0x00);
            }
        } else {
            dest.push(byte);
        }
    }
    dest
}

fn zero_encode(src: &[u8]) -> Vec<u8> {
    let mut dest = Vec::with_capacity(src.len());
    let mut zerocount: u8 = 0;
    let mut i = 0;

    while i < src.len() {
        if src[i] == 0x00 {
            zerocount = zerocount.wrapping_add(1); // increment zero count with wraparound

            // if overflow happens (count wraps to 0), write 0x00 0xff like OpenMetaverse
            if zerocount == 0 {
                dest.push(0x00);
                dest.push(0xff);
                zerocount = 1; // reset to 1 after writing overflow
            }
        } else {
            // flush any accumulated zeros
            if zerocount != 0 {
                dest.push(0x00);
                dest.push(zerocount);
                zerocount = 0;
            }

            dest.push(src[i]);
        }

        i += 1;
    }

    // flush remaining zeros at the end
    if zerocount != 0 {
        dest.push(0x00);
        dest.push(zerocount);
    }

    dest
}

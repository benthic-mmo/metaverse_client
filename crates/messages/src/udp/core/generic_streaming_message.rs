use crate::errors::ParseError;
use crate::packet::{
    header::{Header, PacketFrequency},
    packet_protocol::{Packet, PacketData},
    packet_types::PacketType,
};
/// this is a file for easily creating a new packet.
/// Simply copy this and fill in the data to create a new packet
/// *local_name*    is something like "region_handshake"
/// *PacketName*    is the name of the packet like "RegionHandshake"
/// *id*            is the ID of the packet
///
use std::io::Cursor;

impl Packet {
    pub fn new_generic_streaming_message(
        generic_streaming_message: GenericStreamingMessage,
    ) -> Self {
        Packet {
            header: Header {
                id: 29,
                reliable: false,
                zerocoded: false,
                frequency: PacketFrequency::High,
                ..Default::default()
            },
            body: PacketType::GenericStreamingMessage(Box::new(generic_streaming_message)),
        }
    }
}

/// add your struct fields here
#[derive(Debug, Clone)]
pub struct GenericStreamingMessage {}

impl PacketData for GenericStreamingMessage {
    fn from_bytes(bytes: &[u8]) -> Result<Self, ParseError> {
        let mut cursor = Cursor::new(bytes);
        // handle from bytes
        Ok(GenericStreamingMessage {})
    }
    fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        // push your data into the new vector
        bytes
    }
}

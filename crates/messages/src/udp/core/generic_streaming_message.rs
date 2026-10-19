use crate::errors::ParseError;
use crate::packet::{
    header::{Header, PacketFrequency},
    packet_protocol::{Packet, PacketData},
    packet_types::PacketType,
};
use std::io::Cursor;

impl Packet {
    /// TODO: This is a stub
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

#[derive(Debug, Clone)]
/// TODO: this is a stub
pub struct GenericStreamingMessage {}

impl PacketData for GenericStreamingMessage {
    fn from_bytes(bytes: &[u8]) -> Result<Self, ParseError> {
        let _cursor = Cursor::new(bytes);
        Ok(GenericStreamingMessage {})
    }
    fn to_bytes(&self) -> Vec<u8> {
        Vec::new()
    }
}

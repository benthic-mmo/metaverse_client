use crate::errors::ParseError;
use crate::packet::{
    header::{Header, PacketFrequency},
    packet::{Packet, PacketData},
    packet_types::PacketType,
};
use std::io::Cursor;

impl Packet {
    /// create a new parcel overlay packet
    pub fn new_parcel_overlay(parcel_overlay: ParcelOverlay) -> Self {
        Packet {
            header: Header {
                id: 196,
                reliable: true,
                zerocoded: false,
                frequency: PacketFrequency::Low,
                ..Default::default()
            },
            body: PacketType::ParcelOverlay(Box::new(parcel_overlay)),
        }
    }
}

#[derive(Debug, Clone)]
/// TODO: unimplemented
pub struct ParcelOverlay {}

impl PacketData for ParcelOverlay {
    fn from_bytes(bytes: &[u8]) -> Result<Self, ParseError> {
        let _cursor = Cursor::new(bytes);
        Ok(ParcelOverlay {})
    }
    fn to_bytes(&self) -> Vec<u8> {
        Vec::new()
    }
}

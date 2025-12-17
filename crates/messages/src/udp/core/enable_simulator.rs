use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};

use crate::errors::ParseError;
use crate::packet::{
    header::{Header, PacketFrequency},
    packet::{Packet, PacketData},
    packet_types::PacketType,
};
/// this is a file for easily creating a new packet.
/// Simply copy this and fill in the data to create a new packet
/// *local_name*    is something like "region_handshake"
/// *PacketName*    is the name of the packet like "RegionHandshake"
/// *id*            is the ID of the packet
///
use std::io::{Cursor, Read};
use std::net::Ipv4Addr;

impl Packet {
    pub fn new_enable_simulator(enable_simulator: EnableSimulator) -> Self {
        Packet {
            header: Header {
                id: 151,
                reliable: true,
                zerocoded: false,
                frequency: PacketFrequency::Low,
                ..Default::default()
            },
            body: PacketType::EnableSimulator(Box::new(enable_simulator)),
        }
    }
}

/// add your struct fields here
#[derive(Debug, Clone)]
pub struct EnableSimulator {
    pub handle: u64,
    pub ip: String,
    pub port: u16,
}

impl PacketData for EnableSimulator {
    fn from_bytes(bytes: &[u8]) -> Result<Self, ParseError> {
        let mut cursor = Cursor::new(bytes);
        let handle = cursor.read_u64::<LittleEndian>()?;

        let mut ip_bytes = [0u8; 4];
        cursor.read_exact(&mut ip_bytes)?;
        let ip = String::from_utf8_lossy(&ip_bytes).into_owned();

        let port = cursor.read_u16::<LittleEndian>()?;

        Ok(EnableSimulator { handle, ip, port })
    }
    fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(14);
        bytes.write_u64::<LittleEndian>(self.handle).unwrap();
        bytes.extend_from_slice(&self.ip.as_bytes());
        bytes.write_u16::<LittleEndian>(self.port).unwrap();

        bytes
    }
}

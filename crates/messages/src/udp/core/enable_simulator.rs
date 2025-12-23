use crate::errors::ParseError;
use crate::packet::{
    header::{Header, PacketFrequency},
    packet::{Packet, PacketData},
    packet_types::PacketType,
};
use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use std::io::{Cursor, Read};

impl Packet {
    /// create a new enable simulator packet
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

/// EnableSimulator tells the server that the viewer is ready, and to process and display the
/// region.
#[derive(Debug, Clone)]
pub struct EnableSimulator {
    /// the region handle of the region
    pub handle: u64,
    /// the IP of the ready viewer
    pub ip: String,
    /// the port the viewer is connected to
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
        bytes.extend_from_slice(self.ip.as_bytes());
        bytes.write_u16::<LittleEndian>(self.port).unwrap();

        bytes
    }
}

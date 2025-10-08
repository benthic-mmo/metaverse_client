use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use serde::{Deserialize, Serialize};
use std::io::{self, Cursor, Write};

use crate::packet::{
    errors::PacketError,
    header::{Header, PacketFrequency},
    packet::{Packet, PacketData},
    packet_types::PacketType,
};

impl Packet {
    /// create a new coarse location update packet
    pub fn new_coarse_location_update(coarse_location_update: CoarseLocationUpdate) -> Self {
        Packet {
            header: Header {
                id: 6,
                frequency: PacketFrequency::Medium,
                reliable: false,
                sequence_number: 0,
                appended_acks: false,
                zerocoded: false,
                resent: false,
                ack_list: None,
                size: None,
            },
            body: PacketType::CoarseLocationUpdate(Box::new(coarse_location_update)),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
/// Defines locations of agent dots on the minimap.
pub struct MinimapEntities {
    x: u8,
    y: u8,
    z: u8,
}
impl MinimapEntities {
    /// Converts xyz bytes to minimap entities
    pub fn from_bytes(bytes: &[u8], i: &mut usize) -> io::Result<Self> {
        let cursor = Cursor::new(&bytes[*i..]);
        let x = cursor.get_ref()[0];
        let y = cursor.get_ref()[1];
        let z = cursor.get_ref()[2];
        *i += 3; // Move index forward
        Ok(Self { x, y, z })
    }

    /// converts bytes to minimap entities
    pub fn to_bytes(&self, bytes: &mut [u8], i: &mut usize) -> io::Result<()> {
        let mut cursor = Cursor::new(&mut bytes[*i..]);
        cursor.write_all(&[self.x, self.y, self.z])?;
        *i += 3; // Move index forward
        Ok(())
    }
}
#[derive(Debug, Clone, Serialize, Deserialize)]
/// Struct to contain the ID of you, and the agent you are following
pub struct CoarseLocationUpdate {
    /// The xyz locations of agents
    locations: Vec<MinimapEntities>,
    /// the ID of the user
    you: i16,
    /// the ID of the user you are following
    prey: i16,
}

impl PacketData for CoarseLocationUpdate {
    fn from_bytes(bytes: &[u8]) -> Result<Self, PacketError> {
        let mut cursor = Cursor::new(bytes);
        let location_count = cursor.read_u8()? as usize;
        let mut locations = Vec::with_capacity(location_count);

        for _ in 0..location_count {
            let x = cursor.read_u8()?;
            let y = cursor.read_u8()?;
            let z = cursor.read_u8()?;
            locations.push(MinimapEntities { x, y, z });
        }

        // Deserialize IndexBlock
        let you = cursor.read_i16::<LittleEndian>()?;
        let prey = cursor.read_i16::<LittleEndian>()?;

        Ok(CoarseLocationUpdate {
            locations,
            you,
            prey,
        })
    }

    fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();

        // Serialize LocationBlocks
        bytes.push(self.locations.len() as u8);
        for location in &self.locations {
            bytes.push(location.x);
            bytes.push(location.y);
            bytes.push(location.z);
        }

        // Serialize IndexBlock
        bytes.write_i16::<LittleEndian>(self.you).unwrap();
        bytes.write_i16::<LittleEndian>(self.prey).unwrap();

        bytes
    }
}

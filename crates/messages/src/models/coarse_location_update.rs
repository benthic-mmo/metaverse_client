use super::{
    client_update_data::ClientUpdateData,
    packet::{MessageType, PacketData},
};
use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use futures::future::BoxFuture;
use std::{
    collections::HashMap,
    io::{self, Cursor, Write},
    sync::Arc,
};
use tokio::sync::oneshot::Sender;
use std::sync::Mutex;

/// ID: 6
/// Frequency: Medium

#[derive(Debug)]
pub struct MinimapEntities {
    x: u8,
    y: u8,
    z: u8,
}
impl MinimapEntities {
    pub fn from_bytes(bytes: &[u8], i: &mut usize) -> io::Result<Self> {
        let cursor = Cursor::new(&bytes[*i..]);
        let x = cursor.get_ref()[0];
        let y = cursor.get_ref()[1];
        let z = cursor.get_ref()[2];
        *i += 3; // Move index forward
        Ok(Self { x, y, z })
    }

    pub fn to_bytes(&self, bytes: &mut [u8], i: &mut usize) -> io::Result<()> {
        let mut cursor = Cursor::new(&mut bytes[*i..]);
        cursor.write_all(&[self.x, self.y, self.z])?;
        *i += 3; // Move index forward
        Ok(())
    }
}
#[derive(Debug)]
pub struct CoarseLocationUpdate {
    locations: Vec<MinimapEntities>,
    you: i16,
    prey: i16,
}

impl PacketData for CoarseLocationUpdate {
    fn from_bytes(bytes: &[u8]) -> io::Result<Self> {
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
    fn on_receive(
        &self,
        _: Arc<Mutex<HashMap<u32, Sender<()>>>>,
        _: Arc<Mutex<Vec<ClientUpdateData>>>,
    ) -> BoxFuture<'static, ()> {
        // Dummy implementation for boilerplate
        Box::pin(async move {
            // Implement the actual logic here later
            println!("on_receive is not yet implemented.");
        })
    }
    fn message_type(&self) -> MessageType {
        MessageType::Event
    }
}

use crate::models::header::Header;
use std::io;

#[derive(Debug)]
pub struct Packet<T: PacketData> {
    pub header: Header,
    pub body: T,
}

// this is the trait that allows for serializing and deserializing the packet's data
pub trait PacketData: Sized {
    fn from_bytes(bytes: &[u8]) -> io::Result<Self>;
    fn to_bytes(&self) -> Vec<u8>;
}

impl<T: PacketData> Packet<T> {
    pub fn from_bytes(mut bytes: &[u8]) -> io::Result<Self> {
        let header = Header::try_from_bytes(&mut bytes)?;
        let body = T::from_bytes(bytes)?;

        Ok(Self { header, body })
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend(self.header.to_bytes());
        bytes.extend(self.body.to_bytes());
        bytes
    }
}

pub struct GenericData{
    pub bytes: Vec<u8>
}

impl PacketData for GenericData{
    fn from_bytes(bytes: &[u8]) -> io::Result<Self> {
        Ok(Self{bytes: bytes.to_vec()})
    }
    fn to_bytes(&self) -> Vec<u8> {
        self.bytes.clone()
    }
}

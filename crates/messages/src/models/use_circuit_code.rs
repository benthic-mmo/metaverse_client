use std::io;
use crate::models::header::Header;
use uuid::Uuid;


#[derive(Debug)]
pub struct UseCircuitCodePacket{
    pub header: Header,
    pub circuit_code: CircuitCodeBlock,
}
impl UseCircuitCodePacket{
    pub fn from_bytes(mut bytes: &[u8]) -> io::Result<Self> {
        let header = Header::from_bytes(&mut bytes)?;
        let circuit_code = CircuitCodeBlock::from_bytes(bytes)?;
        Ok(Self { header, circuit_code })
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend(self.header.to_bytes());
        bytes.extend(self.circuit_code.to_bytes());
        bytes
    }
}

#[derive(Debug)]
pub struct CircuitCodeBlock{
    pub code: u32,
    pub session_id: Uuid,
    pub id: Uuid
}

impl CircuitCodeBlock {
    pub fn from_bytes(bytes: &[u8]) -> io::Result<Self>{
        let code = u32::from_le_bytes(bytes[0..4].try_into().unwrap());
        let session_id = Uuid::from_slice(&bytes[4..20]).unwrap();
        let id = Uuid::from_slice(&bytes[20..36]).unwrap();

        Ok(Self {
            code,
            session_id,
            id,
        })
    }
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(36);
        bytes.extend_from_slice(&self.code.to_le_bytes());
        bytes.extend(self.session_id.as_bytes());
        bytes.extend(self.id.as_bytes());
        bytes
    }
}


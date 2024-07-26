use std::io;

use uuid::Uuid;

const HEADER_LENGTH: usize = 10;
const CIRCUIT_CODE_LENGTH: usize = 36;
// const USE_CIRCUIT_CODE_LENGTH: usize = HEADER_LENGTH + CIRCUIT_CODE_LENGTH;

#[derive(Debug)]
pub struct UseCircuitCodePacket{
    header: Header,
    circuit_code: CircuitCodeBlock,
}
impl UseCircuitCodePacket{
    pub fn new(code: u32, session_id: Uuid, id: Uuid) -> Self {
        Self {
            header: Header::new(),
            circuit_code: CircuitCodeBlock::new(code, session_id, id),
        }
    }
    pub async fn from_bytes(mut bytes: &[u8]) -> io::Result<Self> {
        let header = Header::from_bytes(&mut bytes).await?;
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
    code: u32,
    session_id: Uuid,
    id: Uuid
}

impl CircuitCodeBlock {
    pub fn new(code: u32, session_id: Uuid, id: Uuid) -> Self {
        Self { code, session_id, id }
    }
    pub fn from_bytes(bytes: &[u8]) -> io::Result<Self>{
        if bytes.len() < 36 {
            return Err(io::Error::new(io::ErrorKind::InvalidData, "Not enough bytes"));
        }
        let code = u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);
        let session_id = match Uuid::from_slice(&bytes[4..20]) {
            Ok(session_id) => session_id,
            Err(_) => return Err(io::Error::new(io::ErrorKind::InvalidData, "Invalid UUID for session_id")),
        };
        
        let id = match Uuid::from_slice(&bytes[20..36]) {
            Ok(id) => id,
            Err(_) => return Err(io::Error::new(io::ErrorKind::InvalidData, "Invalid UUID for id")),
        };
        Ok(Self { code, session_id, id })
    }
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(36);
        bytes.extend(&self.code.to_le_bytes());
        bytes.extend(self.session_id.as_bytes());
        bytes.extend(self.id.as_bytes());
        bytes
    }
}


#[derive(Debug)]
pub struct Header {
    id: u8,
    frequency: PacketFrequency,
    reliable: bool,
}

impl Header {
    pub fn new() -> Self {
        Self {
            id: 3,
            frequency: PacketFrequency::Low,
            reliable: true,
        }
    }
    pub async fn from_bytes(bytes: &mut &[u8]) -> io::Result<Self> {
        if bytes.len() < 1 {
            return Err(io::Error::new(io::ErrorKind::InvalidData, "Not enough bytes"));
        }
        let id = bytes[0];
        // Skipping frequency and reliable for simplicity
        Ok(Self {
            id,
            frequency: PacketFrequency::Low,
            reliable: true,
        })
    }

    pub fn to_bytes(&self) -> Vec<u8> {        
        let mut bytes = Vec::with_capacity(10);
        bytes.push(self.id);
        bytes.extend(self.frequency.to_bytes(&self));
        bytes.push(if self.reliable { 1u8 } else { 0u8 });
        bytes
     }
}

// Utility function to convert a u16 to a big-endian byte array
fn uint16_to_bytes_big(value: u16) -> [u8; 2] {
    [(value >> 8) as u8, (value & 0xFF) as u8]
}

#[derive(Debug)]
pub enum PacketFrequency {
    High, 
    Medium,
    Low,
}
impl PacketFrequency{
    pub fn to_bytes(&self, header: &Header) -> Vec<u8> {
        let mut bytes = Vec::new();
        match self {
        PacketFrequency::High => {
            // 1 byte ID
            bytes.push(header.id as u8);
        }
        PacketFrequency::Medium => {
            // 2 byte ID
            bytes.push(0xFF);
            bytes.push(header.id as u8);
        }
        PacketFrequency::Low => {
            // 4 byte ID
            bytes.push(0xFF);
            bytes.push(0xFF);
            let id_bytes = uint16_to_bytes_big(header.id as u16);
            bytes.extend_from_slice(&id_bytes);
        }
        };

        bytes

    }
}

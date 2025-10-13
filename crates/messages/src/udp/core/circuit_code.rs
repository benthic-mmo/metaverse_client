use crate::packet::{
    errors::PacketError,
    header::{Header, PacketFrequency},
    packet::{Packet, PacketData},
    packet_types::PacketType,
};
use uuid::Uuid;

impl Packet {
    /// Create a new circuit code packet
    pub fn new_circuit_code(circuit_code_block: CircuitCode) -> Self {
        Packet {
            header: Header {
                id: 3,
                frequency: PacketFrequency::Low,
                reliable: false,
                sequence_number: 0,
                appended_acks: false,
                zerocoded: false,
                resent: false,
                ack_list: None,
                size: None,
            },
            body: PacketType::CircuitCode(Box::new(circuit_code_block)),
        }
    }
}

#[derive(Debug, Clone)]
/// The viewer uses this packet to establigh a circuit connection with a simulator. It is the first
/// message before any other circuit communication is possible.
pub struct CircuitCode {
    /// code of the circuit
    pub code: u32,
    /// ID of the user session
    pub session_id: Uuid,
    /// ID of the circuit
    pub id: Uuid,
}

impl PacketData for CircuitCode {
    fn from_bytes(bytes: &[u8]) -> Result<Self, PacketError> {
        let code = u32::from_le_bytes(bytes[0..4].try_into()?);
        let session_id = Uuid::from_slice(&bytes[4..20])?;
        let id = Uuid::from_slice(&bytes[20..36])?;

        Ok(Self {
            code,
            session_id,
            id,
        })
    }
    fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(36);
        bytes.extend_from_slice(&self.code.to_le_bytes());
        bytes.extend(self.session_id.as_bytes());
        bytes.extend(self.id.as_bytes());
        bytes
    }
}

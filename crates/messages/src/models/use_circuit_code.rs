use crate::models::constants::PacketType;
use crate::models::packet::{Frequency, Header};

use byte::ctx::*;
use byte::*;
use uuid::Uuid;

const HEADER_LENGTH: usize = 10;
const CIRCUIT_CODE_LENGTH: usize = 36;
// const USE_CIRCUIT_CODE_LENGTH: usize = HEADER_LENGTH + CIRCUIT_CODE_LENGTH;

#[derive(Clone, Debug)]
pub struct UseCircuitCode {
    pub has_variable_blocks: bool,
    pub packet_type: PacketType,
    pub header: Header,
    pub circuit_code: CircuitCode,
}

/// Attempts to write the UseCircuitCode to bytes
/// returns how many bytes were written
impl TryWrite<Endian> for UseCircuitCode {
    fn try_write(mut self, bytes: &mut [u8], endian: Endian) -> Result<usize> {
        let _offset = &mut 0;
        // write the header to the packet
        self.header.try_write(bytes, BE).unwrap();

        // add the circuit code after the header
        self.circuit_code.offset = HEADER_LENGTH;
        self.circuit_code.try_write(bytes, endian).unwrap();

        Ok(*_offset)
    }
}

#[derive(Clone, Debug)]
pub struct CircuitCode {
    code: u32,
    id: Uuid,
    session_id: Uuid,
    offset: usize,
}

impl TryWrite<Endian> for CircuitCode {
    fn try_write(mut self, bytes: &mut [u8], endian: Endian) -> Result<usize> {
        let offset = &mut self.offset;
        bytes.write_with::<u32>(offset, self.code, endian)?;

        //TODO: figure out why bytes.write won't let me use the array??
        for byte in self.id.as_bytes() {
            bytes.write::<u8>(offset, *byte)?;
        }
        for byte in self.session_id.as_bytes() {
            bytes.write::<u8>(offset, *byte)?;
        }
        Ok(*offset)
    }
}

/// creates a "use circuit code" object with header and default circuitcode values
/// used in the packet queue to append acks and other relevant header information before the packet
/// is written
pub fn create_use_circuit_code(code: u32, session_id: Uuid, id: Uuid) -> UseCircuitCode {
    UseCircuitCode {
        has_variable_blocks: false,
        packet_type: PacketType::UseCircuitCode,
        header: Header {
            reliable: true,
            resent: false,
            zero_coded: false,
            appended_acks: false,
            sequence: 0,
            id: 3,
            packet_frequency: Frequency::Low,
            ack_list: Vec::new(),
            offset: 0,
        },
        circuit_code: CircuitCode {
            code,
            session_id,
            id,
            // a placeholder
            offset: 0,
        },
    }
}

/// creates the "use circuit code" packet.
pub fn create_use_circuit_code_packet(use_circuit_code: UseCircuitCode) -> Result<Vec<u8>> {
    // the size of the packet without appended acks is the size of the header length + the circuit
    // code length
    let mut length: usize = HEADER_LENGTH + CIRCUIT_CODE_LENGTH;
    // calculate the packet size with the acks
    let ack_list_len = use_circuit_code.header.ack_list.len();
    if ack_list_len > 0 {
        length += ack_list_len * 4 + 1
    }
    // create a vector of the proper size
    let mut bytes = vec![0; length];
    use_circuit_code.try_write(&mut bytes, BE)?;

    // return the written packet
    Ok(bytes)
}

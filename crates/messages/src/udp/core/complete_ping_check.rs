use crate::{
    errors::ParseError,
    packet::{
        header::{Header, PacketFrequency},
        packet::{Packet, PacketData},
        packet_types::PacketType,
    },
};

impl Packet {
    /// create a new complete ping check packet
    pub fn new_complete_ping_check(complete_ping_check: CompletePingCheck) -> Self {
        Packet {
            header: Header {
                id: 2,
                reliable: false,
                resent: false,
                zerocoded: false,
                appended_acks: false,
                sequence_number: 0,
                frequency: PacketFrequency::High,
                ack_list: None,
                size: None,
            },
            body: PacketType::CompletePingCheck(Box::new(complete_ping_check)),
        }
    }
}

#[derive(Debug, Clone)]
/// The complete ping check struct
pub struct CompletePingCheck {
    ///PingID should be the same value received from StartPingCheck, to let the source know which ping was completed.
    pub ping_id: u8,
}

impl PacketData for CompletePingCheck {
    fn from_bytes(bytes: &[u8]) -> Result<Self, ParseError> {
        let ping_id = bytes[0];
        Ok(CompletePingCheck { ping_id })
    }
    fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(5);
        bytes.push(self.ping_id);
        bytes
    }
}

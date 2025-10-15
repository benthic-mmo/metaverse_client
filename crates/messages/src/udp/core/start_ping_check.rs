use crate::{
    errors::ParseError,
    packet::{
        header::{Header, PacketFrequency},
        packet::{Packet, PacketData},
        packet_types::PacketType,
    },
};

impl Packet {
    /// create a new start ping check packet
    pub fn new_start_ping_check(start_ping_check: StartPingCheck) -> Self {
        Packet {
            header: Header {
                id: 1,
                reliable: false,
                resent: false,
                zerocoded: false,
                appended_acks: false,
                sequence_number: 0,
                frequency: PacketFrequency::High,
                ack_list: None,
                size: None,
            },
            body: PacketType::StartPingCheck(Box::new(start_ping_check)),
        }
    }
}

#[derive(Debug, Clone)]
/// Used to measure circuit ping times
pub struct StartPingCheck {
    /// ID of the ping. increased by 1 each time StartPingCheck is sent by the source
    /// After 255 it rolls over back to 0
    pub ping_id: u8,
    /// The sequence number of the most recent message sent by the source
    /// stored as little endian
    pub oldest_unacked: u32,
}

impl PacketData for StartPingCheck {
    fn from_bytes(bytes: &[u8]) -> Result<Self, ParseError> {
        let ping_id = bytes[0];
        let oldest_unacked = u32::from_le_bytes(bytes[1..5].try_into()?);

        Ok(StartPingCheck {
            ping_id,
            oldest_unacked,
        })
    }
    fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(5);
        bytes.push(self.ping_id);
        bytes.extend_from_slice(&self.oldest_unacked.to_le_bytes());
        bytes
    }
}

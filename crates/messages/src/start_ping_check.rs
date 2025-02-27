use super::{
    header::{Header, PacketFrequency},
    packet::{Packet, PacketData},
};
use crate::packet_types::PacketType;

impl Packet {
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
pub struct StartPingCheck {
    pub ping_id: u8,
    pub oldest_unacked: u32,
}

impl PacketData for StartPingCheck {
    fn from_bytes(bytes: &[u8]) -> std::io::Result<Self> {
        let ping_id = bytes[0];
        let oldest_unacked = u32::from_le_bytes(bytes[1..5].try_into().unwrap());

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

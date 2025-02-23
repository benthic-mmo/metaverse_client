use crate::packet_types::PacketType;
use super::{
    header::{Header, PacketFrequency},
    packet::{Packet, PacketData},
};

impl Packet{
    pub fn new_complete_ping_check(complete_ping_check: CompletePingCheck) -> Self{
        Packet{
            header: Header{
                id: 2,
                reliable: false, 
                resent: false, 
                zerocoded: false, 
                appended_acks: false, 
                sequence_number: 0, 
                frequency: PacketFrequency::Low, 
                ack_list: None, 
                size: None,
            },
            body: PacketType::CompletePingCheck(Box::new(complete_ping_check))
        }

    }    
}

#[derive (Debug, Clone)]
pub struct CompletePingCheck{
    pub ping_id: u8,
}

impl PacketData for CompletePingCheck{
    fn from_bytes(bytes: &[u8]) -> std::io::Result<Self>{
        let ping_id = bytes[0];

        Ok(CompletePingCheck {
            ping_id,
        })
    }
    fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(5);
        bytes.push(self.ping_id);
        bytes
    }
}

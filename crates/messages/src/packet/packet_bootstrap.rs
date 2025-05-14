/// this is a file for easily creating a new packet. 
/// Simply copy this and fill in the data to create a new packet 
/// *local_name*    is something like "region_handshake"
/// *PacketName*    is the name of the packet like "RegionHandshake"
/// *id*            is the ID of the packet 
/// 
use std::io::{self, Cursor};
use crate::packet::{
    header::{Header, PacketFrequency},
    packet::{Packet, PacketData},
    packet_types::PacketType,
};

impl Packet{
    pub fn new_*local_name*(*local_name*: *PacketName* ) -> Self{
        Packet{
            header: Header{
                id: *id* ,
                reliable: false, 
                resent: false, 
                zerocoded: false, 
                appended_acks: false, 
                sequence_number: 0, 
                frequency: PacketFrequency::Low, 
                ack_list: None, 
                size: None
            },
            body: PacketType::*PacketName*(Box::new(*local_name*)),
        }
    }
}

/// add your struct fields here
#[derive (Debug, Clone)]
pub struct *PacketName*{

}

impl PacketData for *PacketName* {
    fn from_bytes(bytes: &[u8]) -> io::Result<Self> {
        let mut cursor = Cursor::new(bytes);
        // handle from bytes
        Ok(*PacketName*{
            // Struct fields 
        })
    }
    fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        // push your data into the new vector
        bytes 
    }
} 

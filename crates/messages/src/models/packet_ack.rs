use super::packet::PacketData;

#[derive(Debug)]
pub struct PacketAck {
    pub packet_ids: Vec<u32>,
}

impl PacketData for PacketAck {
    fn from_bytes(bytes: &[u8]) -> std::io::Result<Self> {
        println!("THESE ARE THE BYTES: {:?}", bytes);
        Ok(PacketAck { packet_ids: vec![] })
    }
    fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(5 + self.packet_ids.len() * 4);
        for id in &self.packet_ids {
            bytes.extend_from_slice(&id.to_le_bytes());
        }
        bytes
    }
}

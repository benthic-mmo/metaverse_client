#[derive(Debug, Clone)]
pub enum HeaderConstants {
    AppendedAcks,
    Resent,
    Reliable,
    ZeroCoded,
}

impl HeaderConstants {
    pub fn value(&self) -> u8 {
        match *self {
            HeaderConstants::AppendedAcks => 0x10,
            HeaderConstants::Resent => 0x20,
            HeaderConstants::Reliable => 0x40,
            HeaderConstants::ZeroCoded => 0x80,
        }
    }
}

#[derive(Debug, Clone)]
pub enum PacketType {
    UseCircuitCode = 65539,
}

use crate::models::constants::*;

use byte::ctx::*;
use byte::*;
use std::fmt;

#[derive(Debug, Clone)]
pub struct Packet {
    pub header: Header,
}
impl Packet {
    pub fn as_bytes(&self) -> [u8; 10] {
        let mut bytes: [u8; 10] = [0; 10];
        self.header.clone().try_write(&mut bytes, BE).unwrap();
        bytes
    }
}
impl fmt::Display for Packet {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let header_str = format!("{}", self.header);
        let bytes = self.as_bytes();

        write!(f, "Packet: {{ header: {}, bytes: {:?} }}", header_str, bytes)
    }
}

#[derive(Debug, Clone)]
pub struct Header {
    pub reliable: bool,
    pub resent: bool,
    pub zero_coded: bool,
    pub appended_acks: bool,
    pub sequence: u16,
    pub id: u8,
    pub packet_frequency: Frequency,
    pub ack_list: Vec<u8>,
    pub offset: usize,
}
impl fmt::Display for Header {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Header {{ reliable: {}, resent: {}, zero_coded: {}, appended_acks: {}, sequence: {}, id: {}, packet_frequency: {}, ack_list: {:?}, offset: {} }}",
            self.reliable,
            self.resent,
            self.zero_coded,
            self.appended_acks,
            self.sequence,
            self.id,
            self.packet_frequency,
            self.ack_list,
            self.offset
        )
    }
}

#[derive(Clone, Debug)]
pub enum Frequency {
    Low,
    Medium,
    High,
}
impl fmt::Display for Frequency {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Frequency::Low => write!(f, "Low"),
            Frequency::Medium => write!(f, "Medium"),
            Frequency::High => write!(f, "High"),
        }
    }
}

/// Header layout
/// +-+-+-+-+----+--------+--------+--------+--------+--------+-----...-----+
/// |Z|R|R|A| UN |                                   |        |  Extra      |
/// |E|E|E|C| US |    Sequence number (4 bytes)      | Extra  |  Header     |
/// |R|L|S|K| ED |                                   | (byte) | (N bytes)   |
/// +-+-+-+-+----+--------+--------+--------+--------+--------+-----...-----+
impl TryWrite<Endian> for Header {
    fn try_write(self, bytes: &mut [u8], endian: Endian) -> Result<usize> {
        let offset = &mut 0;

        let mut flags = 0;
        if self.reliable {
            flags |= HeaderConstants::Reliable.value()
        };
        if self.resent {
            flags |= HeaderConstants::Resent.value()
        };
        if self.zero_coded {
            flags |= HeaderConstants::ZeroCoded.value()
        };
        if self.appended_acks {
            flags |= HeaderConstants::AppendedAcks.value()
        };

        bytes.write::<u8>(offset, flags)?;
        bytes.write_with::<u16>(offset, self.sequence, endian)?;
        bytes.write::<u8>(offset, 0)?;
        match self.packet_frequency {
            Frequency::High => {
                bytes.write::<u8>(offset, self.id)?;
            }
            Frequency::Medium => {
                bytes.write::<u8>(offset, 0xFF)?;
                bytes.write::<u8>(offset, self.id)?;
            }
            Frequency::Low => {
                bytes.write::<u8>(offset, 0xFF)?;
                bytes.write::<u8>(offset, 0xFF)?;
                bytes.write::<u16>(offset, self.id.into())?;
            }
        }
        // TODO: WRITE ACK LIST TRY_WRITE
        Ok(*offset)
    }
}



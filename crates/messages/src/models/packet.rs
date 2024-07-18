use crate::models::constants::*;

use byte::ctx::*;
use byte::*;

pub struct Packet {
    pub header: Header,
}
impl Packet {
    pub fn as_bytes(self) -> [u8; 10] {
        let mut bytes: [u8; 10] = [0; 10];
        self.header.try_write(&mut bytes, BE).unwrap();
        bytes
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

#[derive(Clone, Debug)]
pub enum Frequency {
    Low,
    Medium,
    High,
}

use std::io;

// TODO: change the flags to bitflags
pub const MSG_RELIABLE: u8 = 0x01;
pub const MSG_RESENT: u8 = 0x02;
pub const MSG_ZEROCODED: u8 = 0x04;
pub const MSG_APPENDED_ACKS: u8 = 0x08;

#[derive(Debug)]
pub struct Header {
    pub reliable: bool,
    pub resent: bool,
    pub zerocoded: bool,
    pub appended_acks: bool,
    pub sequence_number: u32,
    pub id: u16,
    pub frequency: PacketFrequency,
    pub ack_list: Option<Vec<u32>>,
    pub size: Option<usize>,
}
impl Header {
    pub fn try_from_bytes(bytes: &[u8]) -> Result<Header, std::io::Error> {
        let mut pos = 0;

        let flags = bytes[pos];
        let appended_acks = (flags & MSG_APPENDED_ACKS) != 0;
        let reliable = (flags & MSG_RELIABLE) != 0;
        let resent = (flags & MSG_RESENT) != 0;
        let zerocoded = (flags & MSG_ZEROCODED) != 0;

        pos += 1;
        let sequence_number =
            u32::from_be_bytes([bytes[pos], bytes[pos + 1], bytes[pos + 2], bytes[pos + 3]]);

        pos += 4;

        // Skip the extra byte
        pos += 1;

        let available_bytes = bytes.len() - pos;
        // take at least four bytes from the header
        let slice_length = available_bytes.min(4);

        let (frequency, id, frequency_size) =
            PacketFrequency::from_bytes(&bytes[pos..pos + slice_length], zerocoded)
                .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

        pos += frequency_size;

        let ack_list = if appended_acks {
            let count = bytes[pos];
            pos -= 1;

            let mut acks = Vec::with_capacity(count as usize);

            for _ in 0..count {
                let offset = pos - 3;
                let ack = u32::from_be_bytes([
                    bytes[offset],
                    bytes[offset + 1],
                    bytes[offset + 2],
                    bytes[offset + 3],
                ]);

                acks.push(ack);
                pos -= 4;
            }
            Some(acks)
        } else {
            None
        };

        let header = Header {
            appended_acks,
            reliable,
            resent,
            zerocoded,
            sequence_number,
            frequency,
            id,
            ack_list,
            size: Some(pos),
        };

        Ok(header)
    }
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(10);

        // Add the flags byte
        // TODO fix this
        let mut flags = 0;
        if self.appended_acks {
            flags |= MSG_APPENDED_ACKS;
        }
        if self.reliable {
            flags |= MSG_RELIABLE;
        }
        if self.resent {
            flags |= MSG_RESENT;
        }
        if self.zerocoded {
            flags |= MSG_ZEROCODED;
        }
        bytes.push(flags);

        // Add the sequence number (4 bytes, big-endian)
        bytes.extend_from_slice(&self.sequence_number.to_be_bytes());

        // Add the extra byte
        bytes.push(0);

        // Add the ID and frequency
        bytes.extend_from_slice(&self.frequency.to_bytes(self));

        // Append the ack list if appended_acks is true
        if self.appended_acks {
            if let Some(ref ack_list) = self.ack_list {
                bytes.push(ack_list.len() as u8);
                for ack in ack_list {
                    bytes.extend_from_slice(&ack.to_be_bytes());
                }
            }
        }

        bytes
    }
}

// Utility function to convert a u16 to a big-endian byte array
fn uint16_to_bytes_big(value: u16) -> [u8; 2] {
    [(value >> 8) as u8, (value & 0xFF) as u8]
}

#[derive(Debug)]
pub enum PacketFrequency {
    High,
    Medium,
    Low,
}
impl PacketFrequency {
    pub fn to_bytes(&self, header: &Header) -> Vec<u8> {
        let mut bytes = Vec::new();
        match self {
            PacketFrequency::High => {
                // 1 byte ID
                bytes.push(header.id as u8);
            }
            PacketFrequency::Medium => {
                // 2 byte ID
                bytes.push(0xFF);
                bytes.push(header.id as u8);
            }
            PacketFrequency::Low => {
                // 4 byte ID
                bytes.push(0xFF);
                bytes.push(0xFF);
                let id_bytes = uint16_to_bytes_big(header.id as u16);
                bytes.extend_from_slice(&id_bytes);
            }
        };
        bytes
    }
    pub fn from_bytes(bytes: &[u8], zerocoded: bool) -> io::Result<(Self, u16, usize)> {
        if bytes.is_empty() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Empty PacketFrequency",
            ));
        }

        let id;
        let frequency;
        let size;

        // TODO: THIS WILL BREAK WHEN ACKS ARE APPENDED
        // IF YOU ARE HAVING PROBLEMS PARSING HEADERS THIS IS WHY
        // if there are more than 2 bytes received,
        if bytes.len() >= 2 {
            if bytes[0] == 0xFF && bytes[1] == 0xFF {
                // if the first two bytes are 0xFF
                frequency = PacketFrequency::Low;
                if zerocoded && bytes[2] == 0 {
                    id = bytes[3] as u16
                } else {
                    id = u16::from_be_bytes([bytes[2], bytes[3]])
                };
                size = 4;
                // if they aren't,
            } else {
                frequency = PacketFrequency::Medium;
                id = bytes[1] as u16;
                size = 2;
            }
        }
        // if there was only one byte received, we know for sure it is high
        else {
            frequency = PacketFrequency::High;
            id = bytes[0] as u16;
            size = 1
        }
        Ok((frequency, id, size))
    }
}

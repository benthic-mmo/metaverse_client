use std::io;
// THIS NEEDS A LOT OF HELP 
// GO THROUGH THIS FILE AND FIX 
// THIS IS SUPER MESSY AND MOSTLY COPIED FROM https://raw.githubusercontent.com/openmetaversefoundation/libopenmetaverse/master/OpenMetaverse/_Packets_.cs
// IT WORKS BUT AT WHAT COST

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
}
impl Header {
    pub fn from_bytes(bytes: &mut &[u8]) -> io::Result<Self> {
        if bytes.len() < 10 {
            return Err(io::Error::new(io::ErrorKind::InvalidData, "Not enough bytes"));
        }

        let mut pos = 0;

        let flags = bytes[pos];
        let appended_acks = (flags & MSG_APPENDED_ACKS) != 0;
        let reliable = (flags & MSG_RELIABLE) != 0;
        let resent = (flags & MSG_RESENT) != 0;
        let zerocoded = (flags & MSG_ZEROCODED) != 0;

        pos += 1;
        let sequence_number = u32::from_be_bytes([
            bytes[pos],
            bytes[pos + 1],
            bytes[pos + 2],
            bytes[pos + 3],
        ]);

        pos += 4;

        // Skip the extra byte
        pos += 1;

        let frequency;
        let id;
        // TODO: fix this good lord
        if bytes[pos] == 0xFF {
            pos += 1;
            if bytes[pos] == 0xFF {
                frequency = PacketFrequency::Low;
                pos += 1;
                id = if zerocoded && bytes[pos] == 0 {
                    bytes[pos + 2] as u16
                } else {
                    u16::from_be_bytes([bytes[pos], bytes[pos + 1]])
                };
                pos += 2;
            } else {
                frequency = PacketFrequency::Medium;
                id = bytes[pos] as u16;
                pos += 1;
            }
        } else {
            frequency = PacketFrequency::High;
            id = bytes[pos] as u16;
            pos += 1;
        }
       
        // TODO: fix this also jeez
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
            ack_list
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
impl PacketFrequency{
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
    pub fn from_u8(bytes: &[u8]) -> io::Result<(Self, usize)> {
        if bytes.is_empty() {
            return Err(io::Error::new(io::ErrorKind::InvalidData, "Not enough bytes"));
        }
        let (frequency, consumed) = if bytes[0] != 0xFF {
            (PacketFrequency::High, 1)
        } else if bytes.len() >= 2 && bytes[1] != 0xFF {
            (PacketFrequency::Medium, 2)
        } else if bytes.len() >= 4 {
            (PacketFrequency::Low, 4)
        } else {
            return Err(io::Error::new(io::ErrorKind::InvalidData, "Invalid frequency bytes"));
        };
        Ok((frequency, consumed))
    }
}

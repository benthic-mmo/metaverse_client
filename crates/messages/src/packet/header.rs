use core::fmt;
use std::io;

// TODO: change the flags to bitflags
/// flag for reliable message
pub const MSG_RELIABLE: u8 = 0x40;
/// flag if the packet is resent
pub const MSG_RESENT: u8 = 0x20;
/// flag if the packet is zerocoded
pub const MSG_ZEROCODED: u8 = 0x80;
/// flag if the packet has acks appended
pub const MSG_APPENDED_ACKS: u8 = 0x10;

#[derive(Debug, Clone)]
/// The header for each packet coming from the server
pub struct Header {
    /// if the packet requires an ack to be sent upon receiving
    pub reliable: bool,
    /// if the packet has been resent
    pub resent: bool,
    /// if the packet is zerocoded.
    /// It compresses sequences of zeros in the message in order to reduce network load.
    /// 20, 0, 0, 0, 0, 0, 0, 0, 5 will be zerocoded as
    /// 20, 0, 7, 5
    /// 0 marks when to start inserting zeroes, and the next byte tells how many zeroes to insert.
    pub zerocoded: bool,
    /// If acks are appended to the end of the packet
    pub appended_acks: bool,
    /// Sequence number of the packet
    pub sequence_number: u32,
    /// packet ID used for determinging which type of packet to deserialize the body as
    pub id: u16,
    /// packet frequency, used for grouping packets based on how often they are received
    pub frequency: PacketFrequency,
    /// list of acks sent with the header
    pub ack_list: Option<Vec<u32>>,
    /// the size of the packet
    pub size: Option<usize>,
}
impl Header {
    /// parse the header from incoming packet bytes. Can fail and return an io error.
    pub fn try_from_bytes(bytes: &[u8]) -> Result<Header, std::io::Error> {
        let mut pos = 0;

        let flags = bytes[pos];
        let appended_acks = (flags & MSG_APPENDED_ACKS) != 0;
        let reliable = (flags & MSG_RELIABLE) != 0;
        let resent = (flags & MSG_RESENT) != 0;
        let zerocoded = (flags & MSG_ZEROCODED) != 0;

        // the flags live in one byte
        pos += 1;
        let sequence_number =
            u32::from_be_bytes([bytes[pos], bytes[pos + 1], bytes[pos + 2], bytes[pos + 3]]);
        // the sequence number lives in 4 bytes
        pos += 4;

        let available_bytes = bytes.len() - pos;
        // take at least six bytes from the header
        let slice_length = available_bytes.min(6);

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
        //info!("HEADER: id:{:?}, frequency:{:?}", id, frequency);
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
    /// convert a header to bytes to send as a packet
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
        if self.appended_acks
            && let Some(ref ack_list) = self.ack_list
        {
            bytes.push(ack_list.len() as u8);
            for ack in ack_list {
                bytes.extend_from_slice(&ack.to_be_bytes());
            }
        }

        bytes
    }
}

// Utility function to convert a u16 to a big-endian byte array
fn uint16_to_bytes_big(value: u16) -> [u8; 2] {
    [(value >> 8) as u8, (value & 0xFF) as u8]
}

#[derive(Debug, Clone, Copy, PartialEq)]
/// Frequency of how often the packet is sent
pub enum PacketFrequency {
    /// Frequently sent packet
    High,
    /// moderately frequently sent packet
    Medium,
    /// infrequently sent packet
    Low,
    /// packet sent at a fixed interval
    Fixed,
}
impl fmt::Display for PacketFrequency {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PacketFrequency::High => write!(f, "High"),
            PacketFrequency::Medium => write!(f, "Medium"),
            PacketFrequency::Low => write!(f, "Low"),
            PacketFrequency::Fixed => write!(f, "Fixed"),
        }
    }
}

impl PacketFrequency {
    /// convert the enum to bytes.
    /// Packet frequency can vary in size depending on if it is fixed, low, medium or high.
    /// High is a 1 byte ID
    /// Medium is a 2 byte ID
    /// Low is a 4 byte ID
    /// Fixed is a 6 byte ID.
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
                let id_bytes = uint16_to_bytes_big(header.id);
                bytes.extend_from_slice(&id_bytes);
            }
            PacketFrequency::Fixed => {
                bytes.push(0xFF);
                bytes.push(0xFF);
                bytes.push(0xFF);
                bytes.push(header.id as u8); // THIS IS PROBABLY INCORRECT
            }
        };
        bytes
    }

    /// convert bytes to a header struct
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

        // this match against variable sizes is dangerous and doesn't really give that many
        // benefits
        // this should probably be rewritten so it accepts a fixed size to the end zeroed out, and
        // then does operations on that.
        match bytes.len() {
            2 => {
                frequency = PacketFrequency::High;
                id = bytes[0] as u16;
                size = 1;
            }
            3 => {
                frequency = PacketFrequency::Medium;
                id = bytes[2] as u16;
                size = 3;
            }
            5 | 6 => {
                if bytes[1] == 0xFF && bytes[2] == 0xFF && bytes[3] == 0xFF {
                    frequency = PacketFrequency::Fixed;
                    id = bytes[4] as u16;
                    size = 5;
                } else if bytes[1] == 0xFF && bytes[2] == 0xFF {
                    frequency = PacketFrequency::Low;
                    id = if zerocoded && bytes[3] == 0 {
                        bytes[4] as u16
                    } else {
                        u16::from_be_bytes([bytes[3], bytes[4]])
                    };
                    size = 5;
                } else if bytes[1] == 0xFF {
                    frequency = PacketFrequency::Medium;
                    id = bytes[2] as u16;
                    size = 3; // First 2 bytes considered for Medium frequency
                } else {
                    frequency = PacketFrequency::High;
                    id = bytes[1] as u16;
                    size = 2; // First byte considered for High frequency
                }
            }
            _ => {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    "Unsupported packet length",
                ));
            }
        }
        Ok((frequency, id, size))
    }
}

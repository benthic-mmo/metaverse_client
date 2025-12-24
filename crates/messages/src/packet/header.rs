use byteorder::BigEndian;
use byteorder::ReadBytesExt;
use core::fmt;
use std::io::Cursor;

// TODO: change the flags to bitflags
/// flag for reliable message
pub const MSG_RELIABLE: u8 = 0x40;
/// flag if the packet is resent
pub const MSG_RESENT: u8 = 0x20;
/// flag if the packet is zerocoded
pub const MSG_ZEROCODED: u8 = 0x80;
/// flag if the packet has acks appended
pub const MSG_APPENDED_ACKS: u8 = 0x10;

#[derive(Debug, Clone, Default)]
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
        let mut cursor = Cursor::new(bytes);

        let flags = cursor.read_u8()?;
        let appended_acks = (flags & MSG_APPENDED_ACKS) != 0;
        let reliable = (flags & MSG_RELIABLE) != 0;
        let resent = (flags & MSG_RESENT) != 0;
        let zerocoded = (flags & MSG_ZEROCODED) != 0;

        let sequence_number = cursor.read_u32::<BigEndian>()?;

        // extra byte
        let _extra_info = cursor.read_u8()?;

        // this handles the variable lengths of the frequency and ID.
        let (frequency, id) = match cursor.read_u8()? {
            // if the first byte is 255, it could be fixed, low or medium.
            255 => match cursor.read_u8()? {
                255 => match cursor.read_u8()? {
                    255 => {
                        let fixed = cursor.read_u8()?;
                        (PacketFrequency::Fixed, fixed as u16)
                    }
                    low => {
                        let mut high = cursor.read_u8()?;
                        // hardcoded fix for zerocoded headers
                        if zerocoded && low == 0 && high == 1 {
                            high = cursor.read_u8()?;
                        }

                        let combined: u16 = u16::from_le_bytes([high, low]);
                        (PacketFrequency::Low, combined)
                    }
                },
                medium => (PacketFrequency::Medium, medium as u16),
            },
            // if not, it is a high frequency packet.
            high => (PacketFrequency::High, high as u16),
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
            ack_list: None,
            size: Some(cursor.position() as usize),
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

        bytes
    }
}

// Utility function to convert a u16 to a big-endian byte array
fn uint16_to_bytes_big(value: u16) -> [u8; 2] {
    [(value >> 8) as u8, (value & 0xFF) as u8]
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
/// Frequency of how often the packet is sent
pub enum PacketFrequency {
    /// Frequently sent packet
    High,
    /// moderately frequently sent packet
    Medium,
    /// infrequently sent packet
    Low,
    /// packet sent at a fixed interval
    #[default]
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
}

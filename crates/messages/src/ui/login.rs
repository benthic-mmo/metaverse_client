use std::io::{self, BufRead, Read};

use crate::{
    login::login_xmlrpc::Login,
    packet::{
        header::{Header, PacketFrequency},
        packet::{Packet, PacketData},
        packet_types::PacketType,
    },
};

impl Packet {
    /// create a new login packet
    /// This packet is only used for sending login messages to the server from the UI.
    /// this does not exist in the real packet spec.
    pub fn new_login_packet(login_packet: Login) -> Self {
        Packet {
            header: Header {
                id: 66,
                frequency: PacketFrequency::Fixed,
                reliable: true,
                sequence_number: 0,
                appended_acks: false,
                zerocoded: false,
                resent: false,
                ack_list: None,
                size: None,
            },
            body: PacketType::Login(Box::new(login_packet)),
        }
    }
}

impl PacketData for Login {
    fn from_bytes(bytes: &[u8]) -> io::Result<Self> {
        let mut cursor = std::io::Cursor::new(bytes);

        let read_string = |cursor: &mut std::io::Cursor<&[u8]>| -> io::Result<String> {
            let mut buffer = Vec::new();
            cursor.read_until(0, &mut buffer)?;
            buffer.pop(); // Remove null terminator
            String::from_utf8(buffer).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
        };

        let first = read_string(&mut cursor)?;
        let last = read_string(&mut cursor)?;
        let passwd = read_string(&mut cursor)?;
        let start = read_string(&mut cursor)?;
        let channel = read_string(&mut cursor)?;

        let mut bool_buffer = [0u8; 2];
        cursor.read_exact(&mut bool_buffer)?;
        let agree_to_tos = bool_buffer[0] != 0;
        let read_critical = bool_buffer[1] != 0;

        let url = read_string(&mut cursor)?;

        Ok(Login {
            first,
            last,
            passwd,
            start,
            channel,
            agree_to_tos,
            read_critical,
            url,
        })
    }

    fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend(self.first.as_bytes());
        bytes.push(0);
        bytes.extend(self.last.as_bytes());
        bytes.push(0);
        bytes.extend(self.passwd.as_bytes());
        bytes.push(0);
        bytes.extend(self.start.as_bytes());
        bytes.push(0);
        bytes.extend(self.channel.as_bytes());
        bytes.push(0);
        bytes.push(self.agree_to_tos as u8);
        bytes.push(self.read_critical as u8);
        bytes.extend(self.url.as_bytes());
        bytes.push(0);
        bytes
    }
}

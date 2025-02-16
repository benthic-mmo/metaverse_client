// this is not a packet type that exists in the spec!!
// This is only so that logins can be requested from the UI to the client without having to write a
// lot of extra code around it.
// OpenSimulator's login protocols are weird and bad. :(
use crate::models::header::{Header, PacketFrequency};
use crate::models::packet::{Packet, PacketData};
use futures::future::BoxFuture;
use std::any::Any;
use std::collections::HashMap;
use std::io::{self, BufRead, Read};
use std::sync::Arc;
use std::sync::Mutex;
use tokio::sync::oneshot::Sender;

use super::client_update_data::ClientUpdateData;
use super::packet::MessageType;

#[derive(Debug, Clone)]
pub struct Login {
    pub first: String,
    pub last: String,
    pub passwd: String,
    pub start: String,
    pub channel: String,
    pub agree_to_tos: bool,
    pub read_critical: bool,
    pub url: String,
}

impl PacketData for Login {
    fn from_bytes(bytes: &[u8]) -> io::Result<Self> {
        let mut cursor = std::io::Cursor::new(bytes);

        let read_string = |cursor: &mut std::io::Cursor<&[u8]>| -> io::Result<String> {
            let mut buffer = Vec::new();
            cursor.read_until(0, &mut buffer)?;
            buffer.pop(); // Remove null terminator
            Ok(String::from_utf8(buffer)
                .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?)
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

    fn on_receive(
        &self,
        _: Arc<Mutex<HashMap<u32, Sender<()>>>>,
        _: Arc<Mutex<Vec<ClientUpdateData>>>,
    ) -> BoxFuture<'static, ()> {
        Box::pin(async move {
            println!("Login packet received");
        })
    }

    fn message_type(&self) -> MessageType {
        MessageType::Login
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl Packet {
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
            body: Arc::new(login_packet),
        }
    }
}

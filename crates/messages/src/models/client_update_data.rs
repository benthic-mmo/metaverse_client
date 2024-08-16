use super::packet::Packet;

pub enum ClientUpdateContent {
    Data(DataContent),
    Packet(Packet),
}

pub struct DataContent {
    pub content: String,
}

pub struct ClientUpdateData {
    pub content: ClientUpdateContent,
}

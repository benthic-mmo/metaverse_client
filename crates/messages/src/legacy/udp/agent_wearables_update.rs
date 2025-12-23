use byteorder::{LittleEndian, ReadBytesExt};
use std::io::{Cursor, Read};
use uuid::Uuid;

use crate::{
    errors::ParseError,
    packet::{
        header::{Header, PacketFrequency},
        packet::{Packet, PacketData},
        packet_types::PacketType,
    },
    utils::object_types::WearableType,
};

impl Packet {
    /// Create a new agent wearables update packet
    pub fn new_agent_wearables_update(agent_wearables_update: AgentWearablesUpdate) -> Self {
        Packet {
            header: Header {
                id: 382,
                reliable: true,
                zerocoded: false,
                frequency: PacketFrequency::Low,
                ..Default::default()
            },
            body: PacketType::AgentWearablesUpdate(Box::new(agent_wearables_update)),
        }
    }
}

#[derive(Debug, Clone)]
/// This is legacy code. Was used to send the wearables from the server to the client, but now it
/// just sends dummy data. The current flow is to use the FetchInventoryDescendents2 capability
/// endpoint.
pub struct AgentWearablesUpdate {
    /// the agent ID of the user
    pub agent_id: Uuid,
    /// the ID of the session
    pub session_id: Uuid,
    /// the serial number of the wearables update. Used to prevent wearables from sending out of
    /// order.
    pub serial_number: u32,
    /// the wearables included in the request
    pub wearables: Vec<Wearable>,
}

#[derive(Debug, Clone)]
/// the wearables sent back from the server.
pub struct Wearable {
    /// the ID of the item, used for the inventory
    pub item_id: Uuid,
    /// the ID of the asset, used for retrieving the asset from the asset server endpoint  
    pub asset_id: Uuid,
    /// the type of wearable it is
    pub wearable_type: WearableType,
}

impl PacketData for AgentWearablesUpdate {
    fn from_bytes(bytes: &[u8]) -> Result<Self, ParseError> {
        let mut cursor = Cursor::new(bytes);

        let mut id_bytes = [0u8; 16];
        cursor.read_exact(&mut id_bytes)?;
        let agent_id = Uuid::from_bytes(id_bytes);

        let mut session_bytes = [0u8; 16];
        cursor.read_exact(&mut session_bytes)?;
        let session_id = Uuid::from_bytes(session_bytes);

        let serial_number = cursor.read_u32::<LittleEndian>()?;

        let wearable_count = cursor.read_u8()?;
        let mut wearables = Vec::new();

        for _ in (0..wearable_count).collect::<std::vec::Vec<u8>>() {
            let mut item_id_bytes = [0u8; 16];
            cursor.read_exact(&mut item_id_bytes)?;
            let item_id = Uuid::from_bytes(item_id_bytes);

            let mut asset_id_bytes = [0u8; 16];
            cursor.read_exact(&mut asset_id_bytes)?;
            let asset_id = Uuid::from_bytes(asset_id_bytes);

            let wearable_type = WearableType::from_bytes(cursor.read_u8()?);

            wearables.push(Wearable {
                item_id,
                asset_id,
                wearable_type,
            });
        }

        Ok(AgentWearablesUpdate {
            agent_id,
            session_id,
            serial_number,
            wearables,
        })
    }
    fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(self.agent_id.as_bytes());
        bytes.extend_from_slice(self.session_id.as_bytes());
        bytes.extend_from_slice(&self.serial_number.to_le_bytes());
        bytes.push(self.wearables.len() as u8);
        for wearable in &self.wearables {
            bytes.extend_from_slice(wearable.item_id.as_bytes());
            bytes.extend_from_slice(wearable.asset_id.as_bytes());
            bytes.push(wearable.wearable_type.to_bytes());
        }
        bytes
    }
}

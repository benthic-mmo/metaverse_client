use super::header::PacketFrequency;
use crate::legacy::udp::agent_wearables_request::AgentWearablesRequest;
use crate::legacy::udp::agent_wearables_update::AgentWearablesUpdate;
use crate::packet::errors::PacketError;
use crate::packet::packet::PacketData;
use crate::udp::agent::avatar_appearance::AvatarAppearance;
use crate::udp::core::logout_request::LogoutRequest;
use crate::udp::core::object_update::ObjectUpdate;
use crate::{
    udp::agent::{agent_update::AgentUpdate, coarse_location_update::CoarseLocationUpdate},
    udp::chat::{chat_from_simulator::ChatFromSimulator, chat_from_viewer::ChatFromViewer},
    udp::core::{circuit_code::CircuitCode, complete_agent_movement::CompleteAgentMovementData},
    udp::core::{
        complete_ping_check::CompletePingCheck, disable_simulator::DisableSimulator,
        packet_ack::PacketAck, region_handshake::RegionHandshake,
        region_handshake_reply::RegionHandshakeReply, start_ping_check::StartPingCheck,
    },
    udp::environment::layer_data::LayerData,
};
use std::fmt::Debug;

macro_rules! define_packets {
    ( $( $id:literal [$freq:ident] => $variant:ident ),* $(,)? ) => {
        #[derive(Debug, Clone)]
        #[allow(missing_docs)]
        pub enum PacketType {
            $(
                $variant(Box<$variant>),
            )*
        }

        impl PacketType {
            /// call the PacketType's ToBytes function
            pub fn to_bytes(&self) -> Vec<u8> {
                match self {
                    $(
                        PacketType::$variant(data) => data.to_bytes(),
                    )*
                }
            }
            /// Determine the type of a packet using the ID and frequency
            pub fn from_id(id: u16, frequency: PacketFrequency, bytes: &[u8]) -> Result<Self, PacketError> {
                $(
                    if id == $id && frequency == PacketFrequency::$freq {
                        return Ok(PacketType::$variant(Box::new(PacketData::from_bytes(bytes)?)));
                    }
                )*

                Err(PacketError::InvalidData { id, frequency })
            }
            pub fn variant_name(&self) -> &'static str {
                match self {
                    $(
                        PacketType::$variant(_) => stringify!($variant),
                    )*
                }
            }
        }
    }
}
// The packet type implementation for each packet.
// packets are determined based on their ID and Frequency.
// this macro allows for simple assinging IDs and Frequencies to packets
define_packets! {
    1 [High] => StartPingCheck,
    2 [High] => CompletePingCheck,
    4 [High] => AgentUpdate,
    11 [High] => LayerData,
    12 [High] => ObjectUpdate,

    6 [Medium] => CoarseLocationUpdate,

    3 [Low] => CircuitCode,
    80 [Low] => ChatFromViewer,
    139 [Low] => ChatFromSimulator,
    148 [Low] => RegionHandshake,
    149 [Low] => RegionHandshakeReply,
    152 [Low] => DisableSimulator,
    158 [Low] => AvatarAppearance,
    249 [Low] => CompleteAgentMovementData,
    252 [Low] => LogoutRequest,
    251 [Low] => PacketAck,

    // Legacy packets
    382 [Low] => AgentWearablesUpdate,
    381 [Low] => AgentWearablesRequest
}

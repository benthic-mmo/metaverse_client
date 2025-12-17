use crate::errors::ParseError;
use crate::packet::{
    header::{Header, PacketFrequency},
    packet::{Packet, PacketData},
    packet_types::PacketType,
};
use bitflags::bitflags;
use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
/// this is a file for easily creating a new packet.
/// Simply copy this and fill in the data to create a new packet
/// *local_name*    is something like "region_handshake"
/// *PacketName*    is the name of the packet like "RegionHandshake"
/// *id*            is the ID of the packet
///
use std::io::Cursor;

impl Packet {
    pub fn new_teleport_start(teleport_start: TeleportStart) -> Self {
        Packet {
            header: Header {
                id: 73,
                reliable: true,
                zerocoded: false,
                frequency: PacketFrequency::Low,
                ..Default::default()
            },
            body: PacketType::TeleportStart(Box::new(teleport_start)),
        }
    }
}

/// add your struct fields here
#[derive(Debug, Clone)]
pub struct TeleportStart {
    pub flags: Vec<TeleportFlag>,
}

bitflags! {
    pub struct TeleportBitFlags: u32 {
        const DEFAULT           = 0;
        const SET_HOME_TO_TARGET = 1 << 0;
        const SET_LAST_TO_TARGET = 1 << 1;
        const VIA_LURE          = 1 << 2;
        const VIA_LANDMARK      = 1 << 3;
        const VIA_LOCATION      = 1 << 4;
        const VIA_HOME          = 1 << 5;
        const VIA_TELEHUB       = 1 << 6;
        const VIA_LOGIN         = 1 << 7;
        const VIA_GODLIKE_LURE  = 1 << 8;
        const GODLIKE           = 1 << 9;
        const NINE_ONE_ONE      = 1 << 10;
        const DISABLE_CANCEL    = 1 << 11;
        const VIA_REGION_ID     = 1 << 12;
        const IS_FLYING         = 1 << 13;
        const RESET_HOME        = 1 << 14;
        const FORCE_REDIRECT    = 1 << 15;
        const FINISHED_VIA_LURE = 1 << 26;
        const FINISHED_VIA_NEW_SIM = 1 << 28;
        const FINISHED_VIA_SAME_SIM = 1 << 29;
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TeleportFlag {
    SetHomeToTarget,
    SetLastToTarget,
    Lure,
    Landmark,
    Location,
    Home,
    Telehub,
    Login,
    GodlikeLure,
    Godlike,
    NineOneOne,
    DisableCancel,
    RegionID,
    IsFlying,
    ResetHome,
    ForceRedirect,
    FinishedViaLure,
    FinishedViaNewSim,
    FinishedViaSameSim,
}

impl From<&[TeleportFlag]> for TeleportBitFlags {
    fn from(flags: &[TeleportFlag]) -> Self {
        let mut result = TeleportBitFlags::empty();
        for f in flags {
            result |= match f {
                TeleportFlag::SetHomeToTarget => TeleportBitFlags::SET_HOME_TO_TARGET,
                TeleportFlag::SetLastToTarget => TeleportBitFlags::SET_LAST_TO_TARGET,
                TeleportFlag::Lure => TeleportBitFlags::VIA_LURE,
                TeleportFlag::Landmark => TeleportBitFlags::VIA_LANDMARK,
                TeleportFlag::Location => TeleportBitFlags::VIA_LOCATION,
                TeleportFlag::Home => TeleportBitFlags::VIA_HOME,
                TeleportFlag::Telehub => TeleportBitFlags::VIA_TELEHUB,
                TeleportFlag::Login => TeleportBitFlags::VIA_LOGIN,
                TeleportFlag::GodlikeLure => TeleportBitFlags::VIA_GODLIKE_LURE,
                TeleportFlag::Godlike => TeleportBitFlags::GODLIKE,
                TeleportFlag::NineOneOne => TeleportBitFlags::NINE_ONE_ONE,
                TeleportFlag::DisableCancel => TeleportBitFlags::DISABLE_CANCEL,
                TeleportFlag::RegionID => TeleportBitFlags::VIA_REGION_ID,
                TeleportFlag::IsFlying => TeleportBitFlags::IS_FLYING,
                TeleportFlag::ResetHome => TeleportBitFlags::RESET_HOME,
                TeleportFlag::ForceRedirect => TeleportBitFlags::FORCE_REDIRECT,
                TeleportFlag::FinishedViaLure => TeleportBitFlags::FINISHED_VIA_LURE,
                TeleportFlag::FinishedViaNewSim => TeleportBitFlags::FINISHED_VIA_NEW_SIM,
                TeleportFlag::FinishedViaSameSim => TeleportBitFlags::FINISHED_VIA_SAME_SIM,
            };
        }
        result
    }
}

// Conversion from bitflags -> enum list
impl From<TeleportBitFlags> for Vec<TeleportFlag> {
    fn from(flags: TeleportBitFlags) -> Self {
        let mut result = Vec::new();
        if flags.contains(TeleportBitFlags::SET_HOME_TO_TARGET) {
            result.push(TeleportFlag::SetHomeToTarget);
        }
        if flags.contains(TeleportBitFlags::SET_LAST_TO_TARGET) {
            result.push(TeleportFlag::SetLastToTarget);
        }
        if flags.contains(TeleportBitFlags::VIA_LURE) {
            result.push(TeleportFlag::Lure);
        }
        if flags.contains(TeleportBitFlags::VIA_LANDMARK) {
            result.push(TeleportFlag::Landmark);
        }
        if flags.contains(TeleportBitFlags::VIA_LOCATION) {
            result.push(TeleportFlag::Location);
        }
        if flags.contains(TeleportBitFlags::VIA_HOME) {
            result.push(TeleportFlag::Home);
        }
        if flags.contains(TeleportBitFlags::VIA_TELEHUB) {
            result.push(TeleportFlag::Telehub);
        }
        if flags.contains(TeleportBitFlags::VIA_LOGIN) {
            result.push(TeleportFlag::Login);
        }
        if flags.contains(TeleportBitFlags::VIA_GODLIKE_LURE) {
            result.push(TeleportFlag::GodlikeLure);
        }
        if flags.contains(TeleportBitFlags::GODLIKE) {
            result.push(TeleportFlag::Godlike);
        }
        if flags.contains(TeleportBitFlags::NINE_ONE_ONE) {
            result.push(TeleportFlag::NineOneOne);
        }
        if flags.contains(TeleportBitFlags::DISABLE_CANCEL) {
            result.push(TeleportFlag::DisableCancel);
        }
        if flags.contains(TeleportBitFlags::VIA_REGION_ID) {
            result.push(TeleportFlag::RegionID);
        }
        if flags.contains(TeleportBitFlags::IS_FLYING) {
            result.push(TeleportFlag::IsFlying);
        }
        if flags.contains(TeleportBitFlags::RESET_HOME) {
            result.push(TeleportFlag::ResetHome);
        }
        if flags.contains(TeleportBitFlags::FORCE_REDIRECT) {
            result.push(TeleportFlag::ForceRedirect);
        }
        if flags.contains(TeleportBitFlags::FINISHED_VIA_LURE) {
            result.push(TeleportFlag::FinishedViaLure);
        }
        if flags.contains(TeleportBitFlags::FINISHED_VIA_NEW_SIM) {
            result.push(TeleportFlag::FinishedViaNewSim);
        }
        if flags.contains(TeleportBitFlags::FINISHED_VIA_SAME_SIM) {
            result.push(TeleportFlag::FinishedViaSameSim);
        }
        result
    }
}

impl PacketData for TeleportStart {
    fn from_bytes(bytes: &[u8]) -> Result<Self, ParseError> {
        let mut cursor = Cursor::new(bytes);
        let bitmask = cursor.read_u32::<LittleEndian>()?;
        Ok(TeleportStart {
            flags: TeleportBitFlags::from_bits_truncate(bitmask).into(),
        })
    }

    fn to_bytes(&self) -> Vec<u8> {
        let bitflags: TeleportBitFlags = self.flags.as_slice().into();
        let mut bytes = Vec::with_capacity(4);
        bytes.write_u32::<LittleEndian>(bitflags.bits()).unwrap();
        bytes
    }
}

use serde::{Deserialize, Serialize};

#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ObjectFlag {
    UsePhysics = 0x0000_0001,
    CreateSelected = 0x0000_0002,
    ObjectModify = 0x0000_0004,
    ObjectCopy = 0x0000_0008,
    ObjectAnyOwner = 0x0000_0010,
    ObjectYouOwner = 0x0000_0020,
    Scripted = 0x0000_0040,
    HandleTouch = 0x0000_0080,
    ObjectMove = 0x0000_0100,
    TakesMoney = 0x0000_0200,
    Phantom = 0x0000_0400,
    InventoryEmpty = 0x0000_0800,
    JointHinge = 0x0000_1000,
    JointP2P = 0x0000_2000,
    JointLP2P = 0x0000_4000,
    IncludeInSearch = 0x0000_8000,
    AllowInventoryDrop = 0x0001_0000,
    ObjectTransfer = 0x0002_0000,
    ObjectGroupOwned = 0x0004_0000,
    CameraDecoupled = 0x0010_0000,
    AnimSource = 0x0020_0000,
    CameraSource = 0x0040_0000,
    CastShadows = 0x0080_0000,
    ObjectOwnerModify = 0x1000_0000,
    TemporaryOnRez = 0x2000_0000,
    Temporary = 0x4000_0000,
    ZlibCompressed = 0x8000_0000,
}

impl ObjectFlag {
    pub fn from_bytes(bits: u32) -> Vec<ObjectFlag> {
        let mut flags = Vec::new();
        for &flag in [
            ObjectFlag::UsePhysics,
            ObjectFlag::CreateSelected,
            ObjectFlag::ObjectModify,
            ObjectFlag::ObjectCopy,
            ObjectFlag::ObjectAnyOwner,
            ObjectFlag::ObjectYouOwner,
            ObjectFlag::Scripted,
            ObjectFlag::HandleTouch,
            ObjectFlag::ObjectMove,
            ObjectFlag::TakesMoney,
            ObjectFlag::Phantom,
            ObjectFlag::InventoryEmpty,
            ObjectFlag::JointHinge,
            ObjectFlag::JointP2P,
            ObjectFlag::JointLP2P,
            ObjectFlag::IncludeInSearch,
            ObjectFlag::AllowInventoryDrop,
            ObjectFlag::ObjectTransfer,
            ObjectFlag::ObjectGroupOwned,
            ObjectFlag::CameraDecoupled,
            ObjectFlag::AnimSource,
            ObjectFlag::CameraSource,
            ObjectFlag::CastShadows,
            ObjectFlag::ObjectOwnerModify,
            ObjectFlag::TemporaryOnRez,
            ObjectFlag::Temporary,
            ObjectFlag::ZlibCompressed,
        ]
        .iter()
        {
            if bits & (flag as u32) != 0 {
                flags.push(flag);
            }
        }
        flags
    }
}

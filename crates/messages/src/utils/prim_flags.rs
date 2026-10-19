use serde::{Deserialize, Serialize};

/// Primitive flags
/// this contains information about just about everything.
/// this is used by both ObjectUpdate packets and SceneObjects, though they have different bitmasks
/// between the two.
/// This object unifies both implementations.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PrimFlags {
    /// Object has no collision but still renders
    Phantom,
    /// Object is affected by physics
    Physics,
    /// object is temporary, and will be deleted after about a minute
    TemporaryOnRez,
    /// Object contains at least one running script
    Scripted,
    /// Object has a script that fires on mouse hover
    Touch,
    /// A script in this object can accept money
    Money,
    /// Anyone can drop inventory items into this object's contents
    AllowInventoryDrop,
    /// Object's inventory is empty
    InventoryEmpty,
    /// You are the owner of this object
    ObjectYouOwner,
    /// You are an officer of the group that owns the object
    ObjectYouOfficer,
    /// The object's owner has modify permission
    ObjectOwnerModify,
    /// Object is owned by a group
    ObjectGroupOwned,
    /// Anyone can move this object
    ObjectAnyOwner,
    /// The object can be transferred to another owner
    ObjectTransfer,
    /// You have copy permission on this object
    ObjectCopy,
    /// You have modify permission on this object
    ObjectModify,
    /// You have move permission on this object
    ObjectMove,
    /// Camera is decoupled from the object
    CameraDecoupled,
    /// Object provides a custom animation source
    AnimSource,
    /// Object is configured to die at the edge of a region rather than crossing into the next region
    DieAtEdge,
    /// Object is configured to be returned when it reaches the region edge
    ReturnAtEdge,
    /// object is a sandbox object, with restricted behavior
    Sandbox,
    /// Object should remain selected after creation  
    CreateSelected,

    // Object-update-only flags
    /// Object affects pathfinding for navmesh
    AffectsNavmesh,
    /// Object is a character that can use pathfinding
    Character,
    /// This object handles collision, but doesn't stop movement. Used for creating triggers.
    VolumeDetect,
    /// Object appears in search results
    IncludeInSearch,
    /// Object provides a camera override
    CameraSource,
    /// Object is being piloted by the server  
    ServerAutopilot,
}

impl PrimFlags {
    /// Check a flag against the Prim/XML bit layout.
    pub fn has_flag_prim(bits: u32, flag: Self) -> bool {
        let mask = match flag {
            Self::Phantom => 1 << 0,
            Self::Physics => 1 << 1,
            Self::TemporaryOnRez => 1 << 2,
            Self::Scripted => 1 << 3,
            Self::Touch => 1 << 4,
            Self::Money => 1 << 5,
            Self::AllowInventoryDrop => 1 << 6,
            Self::InventoryEmpty => 1 << 7,
            Self::ObjectYouOwner => 1 << 8,
            Self::ObjectYouOfficer => 1 << 9,
            Self::ObjectOwnerModify => 1 << 10,
            Self::ObjectGroupOwned => 1 << 11,
            Self::ObjectAnyOwner => 1 << 12,
            Self::ObjectTransfer => 1 << 13,
            Self::ObjectCopy => 1 << 14,
            Self::ObjectModify => 1 << 15,
            Self::ObjectMove => 1 << 16,
            Self::CameraDecoupled => 1 << 17,
            Self::AnimSource => 1 << 18,
            Self::DieAtEdge => 1 << 19,
            Self::ReturnAtEdge => 1 << 20,
            Self::Sandbox => 1 << 21,
            Self::CreateSelected => 1 << 22,

            // No Prim/XML equivalent.
            Self::AffectsNavmesh
            | Self::Character
            | Self::VolumeDetect
            | Self::IncludeInSearch
            | Self::CameraSource
            | Self::ServerAutopilot => return false,
        };

        bits & mask != 0
    }

    /// Check a flag against the object-update bit layout.
    pub fn has_flag_object(bits: u32, flag: Self) -> bool {
        let mask = match flag {
            Self::Physics => 0x0000_0001,
            Self::CreateSelected => 0x0000_0002,
            Self::ObjectModify => 0x0000_0004,
            Self::ObjectCopy => 0x0000_0008,
            Self::ObjectAnyOwner => 0x0000_0010,
            Self::ObjectYouOwner => 0x0000_0020,
            Self::Scripted => 0x0000_0040,
            Self::Touch => 0x0000_0080,
            Self::ObjectMove => 0x0000_0100,
            Self::Money => 0x0000_0200,
            Self::Phantom => 0x0000_0400,
            Self::InventoryEmpty => 0x0000_0800,
            Self::AffectsNavmesh => 0x0000_1000,
            Self::Character => 0x0000_2000,
            Self::VolumeDetect => 0x0000_4000,
            Self::IncludeInSearch => 0x0000_8000,
            Self::AllowInventoryDrop => 0x0001_0000,
            Self::ObjectTransfer => 0x0002_0000,
            Self::ObjectGroupOwned => 0x0004_0000,
            Self::CameraDecoupled => 0x0010_0000,
            Self::AnimSource => 0x0020_0000,
            Self::CameraSource => 0x0040_0000,
            Self::ServerAutopilot => 0x0100_0000,
            Self::ObjectOwnerModify => 0x1000_0000,
            Self::TemporaryOnRez => 0x2000_0000,

            // Prim/XML-only flags.
            Self::ObjectYouOfficer | Self::DieAtEdge | Self::ReturnAtEdge | Self::Sandbox => {
                return false;
            }
        };

        bits & mask != 0
    }

    /// Decode all known Prim/XML flags.
    pub fn from_bits_prim(bits: u32) -> Option<Vec<Self>> {
        let flags: Vec<Self> = Self::ALL
            .iter()
            .copied()
            .filter(|flag| Self::has_flag_prim(bits, *flag))
            .collect();
        if flags.is_empty() { None } else { Some(flags) }
    }

    /// Decode all known object-update flags.
    pub fn from_bits_object(bits: u32) -> Option<Vec<Self>> {
        let flags: Vec<Self> = Self::ALL
            .iter()
            .copied()
            .filter(|flag| Self::has_flag_object(bits, *flag))
            .collect();

        if flags.is_empty() { None } else { Some(flags) }
    }
    /// Decode flag from string
    pub fn from_string(value: &str) -> Option<Self> {
        match value {
            "Phantom" => Some(Self::Phantom),
            "Physics" => Some(Self::Physics),
            "TemporaryOnRez" => Some(Self::TemporaryOnRez),
            "Scripted" => Some(Self::Scripted),
            "Touch" => Some(Self::Touch),
            "Money" => Some(Self::Money),
            "AllowInventoryDrop" => Some(Self::AllowInventoryDrop),
            "InventoryEmpty" => Some(Self::InventoryEmpty),
            "ObjectYouOwner" => Some(Self::ObjectYouOwner),
            "ObjectYouOfficer" => Some(Self::ObjectYouOfficer),
            "ObjectOwnerModify" => Some(Self::ObjectOwnerModify),
            "ObjectGroupOwned" => Some(Self::ObjectGroupOwned),
            "ObjectAnyOwner" => Some(Self::ObjectAnyOwner),
            "ObjectTransfer" => Some(Self::ObjectTransfer),
            "ObjectCopy" => Some(Self::ObjectCopy),
            "ObjectModify" => Some(Self::ObjectModify),
            "ObjectMove" => Some(Self::ObjectMove),
            "CameraDecoupled" => Some(Self::CameraDecoupled),
            "AnimSource" => Some(Self::AnimSource),
            "DieAtEdge" => Some(Self::DieAtEdge),
            "ReturnAtEdge" => Some(Self::ReturnAtEdge),
            "Sandbox" => Some(Self::Sandbox),
            "CreateSelected" => Some(Self::CreateSelected),

            "AffectsNavmesh" => Some(Self::AffectsNavmesh),
            "Character" => Some(Self::Character),
            "VolumeDetect" => Some(Self::VolumeDetect),
            "IncludeInSearch" => Some(Self::IncludeInSearch),
            "CameraSource" => Some(Self::CameraSource),
            "ServerAutopilot" => Some(Self::ServerAutopilot),

            "None" => None,
            _ => None,
        }
    }

    const ALL: &'static [Self] = &[
        Self::Phantom,
        Self::Physics,
        Self::TemporaryOnRez,
        Self::Scripted,
        Self::Touch,
        Self::Money,
        Self::AllowInventoryDrop,
        Self::InventoryEmpty,
        Self::ObjectYouOwner,
        Self::ObjectYouOfficer,
        Self::ObjectOwnerModify,
        Self::ObjectGroupOwned,
        Self::ObjectAnyOwner,
        Self::ObjectTransfer,
        Self::ObjectCopy,
        Self::ObjectModify,
        Self::ObjectMove,
        Self::CameraDecoupled,
        Self::AnimSource,
        Self::DieAtEdge,
        Self::ReturnAtEdge,
        Self::Sandbox,
        Self::CreateSelected,
        Self::AffectsNavmesh,
        Self::Character,
        Self::VolumeDetect,
        Self::IncludeInSearch,
        Self::CameraSource,
        Self::ServerAutopilot,
    ];
}

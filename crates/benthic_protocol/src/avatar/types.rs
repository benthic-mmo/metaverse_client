use crate::avatar::skeleton::{DeformName, JointName, Skeleton, SkinJoint};
use glam::{Mat4, Quat, Vec3};
use serde::{Deserialize, Serialize};
use std::{collections::BTreeSet, path::PathBuf};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AttachmentTarget {
    Bone(JointName),
    DeformJoint(DeformName),
    HUDAttachment(AttachPoint),
}

/// Information about the weights of each joint.
/// This corresponds to each vertex in the mesh
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct JointWeight {
    /// The index of the joint the vertex corresponds to
    pub indices: [u8; 4],
    /// How strongly the joint influences the vertex
    pub weights: [f32; 4],
    /// The name of the joint that the weight corresponds to.
    pub joint_name: [SkinJoint; 4],
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct AttachmentData {
    pub attach_point: AttachPoint,
    pub position: Vec3,
    pub rotation: Quat,
    pub scale: Vec3,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
/// Contains skinning information for RenderObjects
pub struct SkinData {
    /// Joint information for the mesh
    pub skeleton: Skeleton,
    /// Weight information for the mesh's joints
    pub weights: Vec<JointWeight>,
    /// Names of all of the joints. In order to correspond to weights and inverse bind matrices.
    pub joint_names: Vec<SkinJoint>,
    /// Inverse bind matrix. Used to determine the shape of the skeleton in relation to the mesh.
    pub inverse_bind_matrices: Vec<Mat4>,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
/// Object that contains the global skeleton for the agent object,  
pub struct AvatarObject {
    /// Path to the RenderObject json on file for the component parts of the outfit
    pub objects: Vec<PathBuf>,
    /// global skeleton that is applied to all objects in the outfit
    pub global_skeleton: Skeleton,
    /// List of joints used by the avatar
    pub used_joints: BTreeSet<SkinJoint>,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
#[repr(u8)]
pub enum AttachPoint {
    #[default]
    Default = 0,
    Chest = 1,
    Head = 2,
    ShoulderLeft = 3,
    ShoulderRight = 4,
    HandLeft = 5,
    HandRight = 6,
    FootLeft = 7,
    FootRight = 8,
    Spine = 9,
    Pelvis = 10,
    Mouth = 11,
    Chin = 12,
    EarLeft = 13,
    EarRight = 14,
    EyeballLeft = 15,
    EyeballRight = 16,
    Nose = 17,
    UpperArmRight = 18,
    LowerArmRight = 19,
    UpperArmLeft = 20,
    LowerArmLeft = 21,
    HipRight = 22,
    UpperLegRight = 23,
    LowerLegRight = 24,
    HipLeft = 25,
    UpperLegLeft = 26,
    LowerLegLeft = 27,
    Stomach = 28,
    PecLeft = 29,
    PecRight = 30,
    HUDCenter2 = 31,
    HUDTopRight = 32,
    HUDTop = 33,
    HUDTopLeft = 34,
    HUDCenter = 35,
    HUDBottomLeft = 36,
    HUDBottom = 37,
    HUDBottomRight = 38,
    Neck = 39,
    Center = 40,
    HandRing1Left = 41,
    HandRing1Right = 42,
    TailBase = 43,
    TailTip = 44,
    WingLeft = 45,
    WingRight = 46,
    FaceJaw = 47,
    FaceEarLeft = 48,
    FaceEarRight = 49,
    FaceEyeLeft = 50,
    FaceEyeRight = 51,
    FaceTongue = 52,
    Groin = 53,
    HindFootLeft = 54,
    HindFootRight = 55,
    Unknown = 99,
}

impl From<AttachPoint> for AttachmentTarget {
    fn from(point: AttachPoint) -> Self {
        match point {
            AttachPoint::Chest => Self::Bone(JointName::Chest),
            AttachPoint::Head => Self::DeformJoint(DeformName::Head),
            AttachPoint::ShoulderLeft => Self::DeformJoint(DeformName::ShoulderLeft),
            AttachPoint::ShoulderRight => Self::DeformJoint(DeformName::ShoulderRight),
            AttachPoint::HandLeft => Self::DeformJoint(DeformName::HandLeft),
            AttachPoint::HandRight => Self::DeformJoint(DeformName::HandRight),
            AttachPoint::FootLeft => Self::DeformJoint(DeformName::FootLeft),
            AttachPoint::FootRight => Self::DeformJoint(DeformName::FootRight),
            AttachPoint::Spine => Self::Bone(JointName::Spine1),
            AttachPoint::Pelvis => Self::Bone(JointName::Pelvis),
            AttachPoint::Mouth => Self::Bone(JointName::FaceTongueTip),
            AttachPoint::Chin => Self::Bone(JointName::FaceJaw),
            AttachPoint::EarLeft => Self::Bone(JointName::FaceEar2Left),
            AttachPoint::EarRight => Self::Bone(JointName::FaceEar2Right),
            AttachPoint::EyeballLeft => Self::Bone(JointName::FaceEyeAltLeft),
            AttachPoint::EyeballRight => Self::Bone(JointName::FaceEyeAltRight),
            AttachPoint::Nose => Self::Bone(JointName::FaceNoseCenter),
            AttachPoint::UpperArmRight => Self::DeformJoint(DeformName::UpperArmRight),
            AttachPoint::LowerArmRight => Self::DeformJoint(DeformName::LowerArmRight),
            AttachPoint::UpperArmLeft => Self::DeformJoint(DeformName::UpperArmLeft),
            AttachPoint::LowerArmLeft => Self::DeformJoint(DeformName::LowerArmLeft),
            AttachPoint::HipRight => Self::DeformJoint(DeformName::HipRight),
            AttachPoint::UpperLegRight => Self::DeformJoint(DeformName::UpperLegRight),
            AttachPoint::LowerLegRight => Self::DeformJoint(DeformName::LowerLegRight),
            AttachPoint::HipLeft => Self::DeformJoint(DeformName::HipLeft),
            AttachPoint::UpperLegLeft => Self::DeformJoint(DeformName::UpperLegLeft),
            AttachPoint::LowerLegLeft => Self::DeformJoint(DeformName::LowerLegLeft),
            AttachPoint::Stomach => Self::DeformJoint(DeformName::Belly),
            AttachPoint::PecLeft => Self::DeformJoint(DeformName::PecLeft),
            AttachPoint::PecRight => Self::DeformJoint(DeformName::PecRight),
            AttachPoint::Neck => Self::DeformJoint(DeformName::Neck),
            AttachPoint::HandRing1Left => Self::Bone(JointName::HandRing1Left),
            AttachPoint::HandRing1Right => Self::Bone(JointName::HandRing1Right),
            AttachPoint::TailBase => Self::Bone(JointName::Tail1),
            AttachPoint::TailTip => Self::Bone(JointName::Tail6),
            AttachPoint::WingLeft => Self::Bone(JointName::Wing1Left),
            AttachPoint::WingRight => Self::Bone(JointName::Wing1Right),
            AttachPoint::FaceJaw => Self::Bone(JointName::FaceJaw),
            AttachPoint::FaceEarLeft => Self::Bone(JointName::FaceEar1Left),
            AttachPoint::FaceEarRight => Self::Bone(JointName::FaceEar1Right),
            AttachPoint::FaceEyeLeft => Self::Bone(JointName::FaceEyeLidUpperLeft),
            AttachPoint::FaceEyeRight => Self::Bone(JointName::FaceEyeLidUpperRight),
            AttachPoint::FaceTongue => Self::Bone(JointName::FaceTongueTip),
            AttachPoint::Groin => Self::Bone(JointName::Groin),
            AttachPoint::HindFootLeft => Self::Bone(JointName::HindLimb4Left),
            AttachPoint::HindFootRight => Self::Bone(JointName::HindLimb4Right),
            AttachPoint::HUDTop => Self::HUDAttachment(AttachPoint::HUDTop),
            AttachPoint::HUDTopRight => Self::HUDAttachment(AttachPoint::HUDTopRight),
            AttachPoint::HUDTopLeft => Self::HUDAttachment(AttachPoint::HUDTopLeft),
            AttachPoint::HUDBottom => Self::HUDAttachment(AttachPoint::HUDBottom),
            AttachPoint::HUDBottomRight => Self::HUDAttachment(AttachPoint::HUDBottomRight),
            AttachPoint::HUDBottomLeft => Self::HUDAttachment(AttachPoint::HUDBottomLeft),
            AttachPoint::HUDCenter => Self::HUDAttachment(AttachPoint::HUDCenter),
            AttachPoint::HUDCenter2 => Self::HUDAttachment(AttachPoint::HUDCenter2),
            AttachPoint::Center => Self::Bone(JointName::Spine2),
            AttachPoint::Default => Self::Bone(JointName::WristRight),
            AttachPoint::Unknown => Self::Bone(JointName::Pelvis),
        }
    }
}
impl AttachPoint {
    pub fn from_i32(value: i32) -> Self {
        match value {
            0 => Self::Default,
            1 => Self::Chest,
            2 => Self::Head,
            3 => Self::ShoulderLeft,
            4 => Self::ShoulderRight,
            5 => Self::HandLeft,
            6 => Self::HandRight,
            7 => Self::FootLeft,
            8 => Self::FootRight,
            9 => Self::Spine,
            10 => Self::Pelvis,
            11 => Self::Mouth,
            12 => Self::Chin,
            13 => Self::EarLeft,
            14 => Self::EarRight,
            15 => Self::EyeballLeft,
            16 => Self::EyeballRight,
            17 => Self::Nose,
            18 => Self::UpperArmRight,
            19 => Self::LowerArmRight,
            20 => Self::UpperArmLeft,
            21 => Self::LowerArmLeft,
            22 => Self::HipRight,
            23 => Self::UpperLegRight,
            24 => Self::LowerLegRight,
            25 => Self::HipLeft,
            26 => Self::UpperLegLeft,
            27 => Self::LowerLegLeft,
            28 => Self::Stomach,
            29 => Self::PecLeft,
            30 => Self::PecRight,
            31 => Self::HUDCenter2,
            32 => Self::HUDTopRight,
            33 => Self::HUDTop,
            34 => Self::HUDTopLeft,
            35 => Self::HUDCenter,
            36 => Self::HUDBottomLeft,
            37 => Self::HUDBottom,
            38 => Self::HUDBottomRight,
            39 => Self::Neck,
            40 => Self::Center,
            41 => Self::HandRing1Left,
            42 => Self::HandRing1Right,
            43 => Self::TailBase,
            44 => Self::TailTip,
            45 => Self::WingLeft,
            46 => Self::WingRight,
            47 => Self::FaceJaw,
            48 => Self::FaceEarLeft,
            49 => Self::FaceEarRight,
            50 => Self::FaceEyeLeft,
            51 => Self::FaceEyeRight,
            52 => Self::FaceTongue,
            53 => Self::Groin,
            54 => Self::HindFootLeft,
            55 => Self::HindFootRight,
            _ => Self::Unknown,
        }
    }
}

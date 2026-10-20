use glam::Mat4;
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use strum_macros::{Display, EnumString};
use uuid::Uuid;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
/// Definitions for the avatar's skeleton  
pub struct Skeleton {
    /// The joints that make up the skeleton
    pub joints: IndexMap<SkinJoint, Joint>,
    /// The root bone
    /// this can contain many joints, if the object has a partial skeleton.
    pub root: Vec<SkinJoint>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
/// Defines the joint that makes up the skeleton
pub struct Joint {
    /// The name of the joint
    pub name: SkinJoint,
    /// The global transforms applied to the joint
    pub global_transforms: Vec<Transform>,
    /// The transforms applied to the joint based on its parent and child relationship.
    pub local_transforms: Vec<Transform>,
    /// The position in the skeleton's vec of the joint's children
    pub children: Vec<SkinJoint>,
    /// The position in the skeleton's vec of the joint's parent
    pub parent: Option<SkinJoint>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
/// This stores the transform that each joint has
/// This is not just a vector of Mat4s to solve a very specific issue.
/// The default skeleton is generated based on the joint transforms of all of the objects in the
/// outfit. If I have a full body override that changes the scale and transform of some of the
/// joints, clothing should be applied based on that global sekeleton's transforms. However you can
/// also apply clothing that overrides the full body's jont transforms, creating a situation where
/// you have transforms from your body like you would expect, combined with transforms coming from
/// your shirt that you would not expect. This can cause nondeterministic behavior if you apply
/// clothes that don't share the same base skeleton. This will be used to allow users to select
/// which item they want to use for their skeleton deforms.
pub struct Transform {
    /// Name of the object that the transform is coming from
    pub name: String,
    /// ID of the object the transform is coming from
    pub id: Uuid,
    /// the transform associated with the object
    pub transform: Mat4,
    /// the number of outfit objects that contain this exact transform. The transform with the
    /// highest number of votes will be applied.
    pub rank: usize,
}

#[derive(Debug, Clone, Default, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
pub enum SkinJoint {
    #[default]
    None,
    Deform(DeformName),
    Joint(JointName),
}
impl std::fmt::Display for SkinJoint {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SkinJoint::Joint(joint) => write!(f, "Joint::{joint}"),
            SkinJoint::Deform(deform) => write!(f, "Deform::{deform}"),
            SkinJoint::None => write!(f, "None"),
        }
    }
}
impl serde::Serialize for SkinJoint {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            SkinJoint::Joint(joint) => serializer.serialize_str(&format!("Joint::{joint}")),
            SkinJoint::Deform(deform) => serializer.serialize_str(&format!("Deform::{deform}")),
            SkinJoint::None => serializer.serialize_str("None"),
        }
    }
}
impl<'de> serde::Deserialize<'de> for SkinJoint {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        s.parse().map_err(serde::de::Error::custom)
    }
}
impl std::str::FromStr for SkinJoint {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s == "None" {
            return Ok(SkinJoint::None);
        }

        let name = s
            .strip_prefix("Joint::")
            .or_else(|| s.strip_prefix("Deform::"))
            .ok_or_else(|| format!("Invalid SkinJoint key: {s}"))?;

        SkinJoint::resolve_joint_and_deform(name)
            .ok_or_else(|| format!("Unknown skin joint: {name}"))
    }
}

#[derive(
    Debug,
    Clone,
    Default,
    Serialize,
    Deserialize,
    EnumString,
    Copy,
    Eq,
    PartialEq,
    Hash,
    PartialOrd,
    Ord,
    Display,
)]
/// This is used to separate out the collision entities from the skeleton.
/// These get passed in as joints, despite not being joints in the default skeleton.
pub enum DeformName {
    #[strum(serialize = "BUTT")]
    Butt,
    // Root
    #[default]
    #[strum(serialize = "PELVIS")]
    Pelvis,
    #[strum(serialize = "LOWER_BACK")]
    LowerBack,
    #[strum(serialize = "BELLY")]
    Belly,
    #[strum(serialize = "UPPER_BACK")]
    UpperBack,
    #[strum(serialize = "LEFT_HANDLE")]
    HandleLeft,
    #[strum(serialize = "RIGHT_HANDLE")]
    HandleRight,
    #[strum(serialize = "CHEST")]
    Chest,
    #[strum(serialize = "NECK")]
    Neck,
    #[strum(serialize = "HEAD")]
    Head,
    #[strum(serialize = "L_CLAVICLE")]
    ClavicleLeft,
    #[strum(serialize = "R_CLAVICLE")]
    ClavicleRight,
    #[strum(serialize = "SHOULDER_LEFT")]
    ShoulderLeft,
    #[strum(serialize = "SHOULDER_RIGHT")]
    ShoulderRight,
    #[strum(serialize = "LEFT_PEC")]
    PecLeft,
    #[strum(serialize = "RIGHT_PEC")]
    PecRight,
    #[strum(serialize = "L_UPPER_ARM")]
    UpperArmLeft,
    #[strum(serialize = "L_LOWER_ARM")]
    LowerArmLeft,
    #[strum(serialize = "R_UPPER_ARM")]
    UpperArmRight,
    #[strum(serialize = "R_LOWER_ARM")]
    LowerArmRight,
    #[strum(serialize = "L_HAND")]
    HandLeft,
    #[strum(serialize = "R_HAND")]
    HandRight,
    #[strum(serialize = "L_UPPER_LEG")]
    UpperLegLeft,
    #[strum(serialize = "L_LOWER_LEG")]
    LowerLegLeft,
    #[strum(serialize = "R_UPPER_LEG")]
    UpperLegRight,
    #[strum(serialize = "R_LOWER_LEG")]
    LowerLegRight,
    #[strum(serialize = "HIP_LEFT")]
    HipLeft,
    #[strum(serialize = "HIP_RIGHT")]
    HipRight,
    #[strum(serialize = "L_FOOT")]
    FootLeft,
    #[strum(serialize = "R_FOOT")]
    FootRight,
    #[strum(serialize = "FOOT_LEFT")]
    FootLeft2,
    #[strum(serialize = "FOOT_RIGHT")]
    FootRight2,
    #[strum(serialize = "KNEE_LEFT")]
    UpperKneeLeft,
    #[strum(serialize = "KNEE_RIGHT")]
    UpperKneeRight,
    #[strum(serialize = "ANKLE_LEFT")]
    AnkleLeft,
    #[strum(serialize = "ANKLE_RIGHT")]
    AnkleRight,
}

#[allow(missing_docs)]
#[repr(u16)]
#[derive(
    EnumString,
    Display,
    Debug,
    Hash,
    PartialEq,
    Eq,
    Clone,
    Default,
    Copy,
    Serialize,
    Deserialize,
    Ord,
    PartialOrd,
)]
/// This is used to convert all of the joint strings into a usable enum. This is to allow for
/// easier usage, potentially renaming joints on export, and avoid checking string equality
/// the strum values are for the actual names of the Bento skeleton.
pub enum JointName {
    #[default]
    #[strum(serialize = "mPelvis")]
    Pelvis,

    #[strum(serialize = "mSpine1")]
    Spine1,
    #[strum(serialize = "mSpine2")]
    Spine2,
    #[strum(serialize = "mSpine3")]
    Spine3,
    #[strum(serialize = "mSpine4")]
    Spine4,

    #[strum(serialize = "mChest")]
    Chest,
    #[strum(serialize = "mTorso")]
    Torso,
    #[strum(serialize = "mGroin")]
    Groin,

    #[strum(serialize = "mNeck")]
    Neck,
    #[strum(serialize = "mHead")]
    Head,
    #[strum(serialize = "mSkull")]
    Skull,

    // Face
    #[strum(serialize = "mFaceRoot")]
    FaceRoot,
    #[strum(serialize = "mFaceForeheadCenter")]
    FaceForeheadCenter,
    #[strum(serialize = "mFaceForeheadLeft")]
    FaceForeheadLeft,
    #[strum(serialize = "mFaceForeheadRight")]
    FaceForeheadRight,
    #[strum(serialize = "mFaceEyebrowOuterLeft")]
    FaceEyebrowOuterLeft,
    #[strum(serialize = "mFaceEyebrowCenterLeft")]
    FaceEyebrowCenterLeft,
    #[strum(serialize = "mFaceEyebrowInnerLeft")]
    FaceEyebrowInnerLeft,
    #[strum(serialize = "mFaceEyebrowOuterRight")]
    FaceEyebrowOuterRight,
    #[strum(serialize = "mFaceEyebrowCenterRight")]
    FaceEyebrowCenterRight,
    #[strum(serialize = "mFaceEyebrowInnerRight")]
    FaceEyebrowInnerRight,
    #[strum(serialize = "mFaceEyecornerInnerLeft")]
    FaceEyecornerInnerLeft,
    #[strum(serialize = "mFaceEyecornerInnerRight")]
    FaceEyecornerInnerRight,
    #[strum(serialize = "mFaceEyeLidUpperLeft")]
    FaceEyeLidUpperLeft,
    #[strum(serialize = "mFaceEyeLidLowerLeft")]
    FaceEyeLidLowerLeft,
    #[strum(serialize = "mFaceEyeLidUpperRight")]
    FaceEyeLidUpperRight,
    #[strum(serialize = "mFaceEyeLidLowerRight")]
    FaceEyeLidLowerRight,
    #[strum(serialize = "mFaceCheekUpperLeft")]
    FaceCheekUpperLeft,
    #[strum(serialize = "mFaceCheekLowerLeft")]
    FaceCheekLowerLeft,
    #[strum(serialize = "mFaceCheekUpperRight")]
    FaceCheekUpperRight,
    #[strum(serialize = "mFaceCheekLowerRight")]
    FaceCheekLowerRight,
    #[strum(serialize = "mFaceNoseBridge")]
    FaceNoseBridge,
    #[strum(serialize = "mFaceNoseCenter")]
    FaceNoseCenter,
    #[strum(serialize = "mFaceNoseBase")]
    FaceNoseBase,
    #[strum(serialize = "mFaceNoseLeft")]
    FaceNoseLeft,
    #[strum(serialize = "mFaceNoseRight")]
    FaceNoseRight,
    #[strum(serialize = "mFaceLipUpperCenter")]
    FaceLipUpperCenter,
    #[strum(serialize = "mFaceLipUpperLeft")]
    FaceLipUpperLeft,
    #[strum(serialize = "mFaceLipCornerLeft")]
    FaceLipCornerLeft,
    #[strum(serialize = "mFaceLipUpperRight")]
    FaceLipUpperRight,
    #[strum(serialize = "mFaceLipCornerRight")]
    FaceLipCornerRight,
    #[strum(serialize = "mFaceLipLowerCenter")]
    FaceLipLowerCenter,
    #[strum(serialize = "mFaceLipLowerLeft")]
    FaceLipLowerLeft,
    #[strum(serialize = "mFaceLipLowerRight")]
    FaceLipLowerRight,
    #[strum(serialize = "mFaceTongueBase")]
    FaceTongueBase,
    #[strum(serialize = "mFaceTongueTip")]
    FaceTongueTip,
    #[strum(serialize = "mFaceTeethUpper")]
    FaceTeethUpper,
    #[strum(serialize = "mFaceTeethLower")]
    FaceTeethLower,
    #[strum(serialize = "mFaceJaw")]
    FaceJaw,
    #[strum(serialize = "mFaceJawShaper")]
    FaceJawShaper,
    #[strum(serialize = "mFaceChin")]
    FaceChin,
    #[strum(serialize = "mFaceEar1Left")]
    FaceEar1Left,
    #[strum(serialize = "mFaceEar2Left")]
    FaceEar2Left,
    #[strum(serialize = "mFaceEar1Right")]
    FaceEar1Right,
    #[strum(serialize = "mFaceEar2Right")]
    FaceEar2Right,
    #[strum(serialize = "mFaceEyeAltLeft")]
    FaceEyeAltLeft,
    #[strum(serialize = "mFaceEyeAltRight")]
    FaceEyeAltRight,

    // Eyes
    #[strum(serialize = "mEyeLeft")]
    EyeLeft,
    #[strum(serialize = "mEyeRight")]
    EyeRight,

    #[strum(serialize = "mShoulderLeft")]
    ShoulderLeft,
    #[strum(serialize = "mShoulderRight")]
    ShoulderRight,
    #[strum(serialize = "mCollarLeft")]
    CollarLeft,
    #[strum(serialize = "mCollarRight")]
    CollarRight,

    #[strum(serialize = "mArmLeft")]
    ArmLeft,
    #[strum(serialize = "mArmRight")]
    ArmRight,
    #[strum(serialize = "mElbowLeft")]
    ElbowLeft,
    #[strum(serialize = "mElbowRight")]
    ElbowRight,
    #[strum(serialize = "mForearmLeft")]
    ForearmLeft,
    #[strum(serialize = "mForearmRight")]
    ForearmRight,
    #[strum(serialize = "mWristLeft")]
    WristLeft,
    #[strum(serialize = "mWristRight")]
    WristRight,

    // Wings
    #[strum(serialize = "mWingsRoot")]
    WingsRoot,
    #[strum(serialize = "mWing1Left")]
    Wing1Left,
    #[strum(serialize = "mWing2Left")]
    Wing2Left,
    #[strum(serialize = "mWing3Left")]
    Wing3Left,
    #[strum(serialize = "mWing4Left")]
    Wing4Left,
    #[strum(serialize = "mWing4FanLeft")]
    Wing4FanLeft,
    #[strum(serialize = "mWing1Right")]
    Wing1Right,
    #[strum(serialize = "mWing2Right")]
    Wing2Right,
    #[strum(serialize = "mWing3Right")]
    Wing3Right,
    #[strum(serialize = "mWing4Right")]
    Wing4Right,
    #[strum(serialize = "mWing4FanRight")]
    Wing4FanRight,

    #[strum(serialize = "mHandLeft")]
    HandLeft,
    #[strum(serialize = "mHandRight")]
    HandRight,

    #[strum(serialize = "mHandIndex1Left")]
    HandIndex1Left,
    #[strum(serialize = "mHandIndex2Left")]
    HandIndex2Left,
    #[strum(serialize = "mHandIndex3Left")]
    HandIndex3Left,

    #[strum(serialize = "mHandMiddle1Left")]
    HandMiddle1Left,
    #[strum(serialize = "mHandMiddle2Left")]
    HandMiddle2Left,
    #[strum(serialize = "mHandMiddle3Left")]
    HandMiddle3Left,

    #[strum(serialize = "mHandRing1Left")]
    HandRing1Left,
    #[strum(serialize = "mHandRing2Left")]
    HandRing2Left,
    #[strum(serialize = "mHandRing3Left")]
    HandRing3Left,

    #[strum(serialize = "mHandPinky1Left")]
    HandPinky1Left,
    #[strum(serialize = "mHandPinky2Left")]
    HandPinky2Left,
    #[strum(serialize = "mHandPinky3Left")]
    HandPinky3Left,

    #[strum(serialize = "mHandIndex1Right")]
    HandIndex1Right,
    #[strum(serialize = "mHandIndex2Right")]
    HandIndex2Right,
    #[strum(serialize = "mHandIndex3Right")]
    HandIndex3Right,

    #[strum(serialize = "mHandMiddle1Right")]
    HandMiddle1Right,
    #[strum(serialize = "mHandMiddle2Right")]
    HandMiddle2Right,
    #[strum(serialize = "mHandMiddle3Right")]
    HandMiddle3Right,

    #[strum(serialize = "mHandRing1Right")]
    HandRing1Right,
    #[strum(serialize = "mHandRing2Right")]
    HandRing2Right,
    #[strum(serialize = "mHandRing3Right")]
    HandRing3Right,

    #[strum(serialize = "mHandPinky1Right")]
    HandPinky1Right,
    #[strum(serialize = "mHandPinky2Right")]
    HandPinky2Right,
    #[strum(serialize = "mHandPinky3Right")]
    HandPinky3Right,

    #[strum(serialize = "mHandThumb1Left")]
    HandThumb1Left,
    #[strum(serialize = "mHandThumb2Left")]
    HandThumb2Left,
    #[strum(serialize = "mHandThumb3Left")]
    HandThumb3Left,
    #[strum(serialize = "mHandThumb1Right")]
    HandThumb1Right,
    #[strum(serialize = "mHandThumb2Right")]
    HandThumb2Right,
    #[strum(serialize = "mHandThumb3Right")]
    HandThumb3Right,

    // Tail
    #[strum(serialize = "mTail1")]
    Tail1,
    #[strum(serialize = "mTail2")]
    Tail2,
    #[strum(serialize = "mTail3")]
    Tail3,
    #[strum(serialize = "mTail4")]
    Tail4,
    #[strum(serialize = "mTail5")]
    Tail5,
    #[strum(serialize = "mTail6")]
    Tail6,

    // Hind Limbs
    #[strum(serialize = "mHindLimbsRoot")]
    HindLimbsRoot,
    #[strum(serialize = "mHindLimb1Left")]
    HindLimb1Left,
    #[strum(serialize = "mHindLimb2Left")]
    HindLimb2Left,
    #[strum(serialize = "mHindLimb3Left")]
    HindLimb3Left,
    #[strum(serialize = "mHindLimb4Left")]
    HindLimb4Left,
    #[strum(serialize = "mHindLimb1Right")]
    HindLimb1Right,
    #[strum(serialize = "mHindLimb2Right")]
    HindLimb2Right,
    #[strum(serialize = "mHindLimb3Right")]
    HindLimb3Right,
    #[strum(serialize = "mHindLimb4Right")]
    HindLimb4Right,

    #[strum(serialize = "mHipLeft")]
    HipLeft,
    #[strum(serialize = "mHipRight")]
    HipRight,
    #[strum(serialize = "mThighLeft")]
    ThighLeft,
    #[strum(serialize = "mThighRight")]
    ThighRight,
    #[strum(serialize = "mKneeLeft")]
    KneeLeft,
    #[strum(serialize = "mKneeRight")]
    KneeRight,
    #[strum(serialize = "mShinLeft")]
    ShinLeft,
    #[strum(serialize = "mShinRight")]
    ShinRight,

    #[strum(serialize = "mAnkleLeft")]
    AnkleLeft,
    #[strum(serialize = "mAnkleRight")]
    AnkleRight,

    #[strum(serialize = "mFootLeft")]
    FootLeft,
    #[strum(serialize = "mFootRight")]
    FootRight,
    #[strum(serialize = "mHeelLeft")]
    HeelLeft,
    #[strum(serialize = "mHeelRight")]
    HeelRight,
    #[strum(serialize = "mToeLeft")]
    ToeLeft,
    #[strum(serialize = "mToeRight")]
    ToeRight,
}

use once_cell::sync::Lazy;
use std::collections::HashMap;

static BVH_JOINT_ALIASES: Lazy<HashMap<&'static str, SkinJoint>> = Lazy::new(|| {
    let mut m = HashMap::new();

    // Root
    m.insert("hip", SkinJoint::Joint(JointName::Pelvis));
    m.insert("abdomen", SkinJoint::Joint(JointName::Torso));
    m.insert("chest", SkinJoint::Joint(JointName::Chest));
    m.insert("neck", SkinJoint::Joint(JointName::Neck));
    m.insert("head", SkinJoint::Joint(JointName::Head));

    // Left arm
    m.insert("lCollar", SkinJoint::Joint(JointName::CollarLeft));
    m.insert("lShldr", SkinJoint::Joint(JointName::ShoulderLeft));
    m.insert("lForeArm", SkinJoint::Joint(JointName::ElbowLeft));
    m.insert("lHand", SkinJoint::Joint(JointName::HandLeft));

    // Right arm
    m.insert("rCollar", SkinJoint::Joint(JointName::CollarRight));
    m.insert("rShldr", SkinJoint::Joint(JointName::ShoulderRight));
    m.insert("rForeArm", SkinJoint::Joint(JointName::ElbowRight));
    m.insert("rHand", SkinJoint::Joint(JointName::HandRight));

    // Left leg
    m.insert("lThigh", SkinJoint::Joint(JointName::HipLeft));
    m.insert("lShin", SkinJoint::Joint(JointName::KneeLeft));
    m.insert("lFoot", SkinJoint::Joint(JointName::FootLeft));

    // Right leg
    m.insert("rThigh", SkinJoint::Joint(JointName::HipRight));
    m.insert("rShin", SkinJoint::Joint(JointName::KneeRight));
    m.insert("rFoot", SkinJoint::Joint(JointName::FootRight));

    m
});

static QUATERNIUS_JOINT_ALIASES: Lazy<HashMap<&'static str, SkinJoint>> = Lazy::new(|| {
    let mut m = HashMap::new();

    // Spine
    m.insert("pelvis", SkinJoint::Joint(JointName::Pelvis));
    m.insert("spine_01", SkinJoint::Joint(JointName::Torso));
    m.insert("spine_02", SkinJoint::Joint(JointName::Spine2));
    m.insert("spine_03", SkinJoint::Joint(JointName::Chest));

    // Neck & Head
    m.insert("neck_01", SkinJoint::Joint(JointName::Head));
    m.insert("Head", SkinJoint::Joint(JointName::Skull));

    // Left Arm
    m.insert("upperarm_l", SkinJoint::Joint(JointName::ShoulderLeft));
    m.insert("lowerarm_l", SkinJoint::Joint(JointName::ElbowLeft));
    m.insert("hand_l", SkinJoint::Joint(JointName::WristLeft));
    m.insert("clavicle_l", SkinJoint::Joint(JointName::CollarLeft));

    // Right Arm
    m.insert("upperarm_r", SkinJoint::Joint(JointName::ShoulderRight));
    m.insert("lowerarm_r", SkinJoint::Joint(JointName::ElbowRight));
    m.insert("hand_r", SkinJoint::Joint(JointName::WristRight));
    m.insert("clavicle_r", SkinJoint::Joint(JointName::CollarRight));

    // Left Leg
    m.insert("thigh_l", SkinJoint::Joint(JointName::HipLeft));
    m.insert("calf_l", SkinJoint::Joint(JointName::KneeLeft));
    m.insert("foot_l", SkinJoint::Joint(JointName::FootLeft));

    // Right Leg
    m.insert("thigh_r", SkinJoint::Joint(JointName::HipRight));
    m.insert("calf_r", SkinJoint::Joint(JointName::KneeRight));
    m.insert("foot_r", SkinJoint::Joint(JointName::FootRight));

    // Left Fingers
    m.insert("thumb_01_l", SkinJoint::Joint(JointName::HandThumb1Left));
    m.insert("thumb_02_l", SkinJoint::Joint(JointName::HandThumb2Left));
    m.insert("thumb_03_l", SkinJoint::Joint(JointName::HandThumb3Left));

    m.insert("index_01_l", SkinJoint::Joint(JointName::HandIndex1Left));
    m.insert("index_02_l", SkinJoint::Joint(JointName::HandIndex2Left));
    m.insert("index_03_l", SkinJoint::Joint(JointName::HandIndex3Left));

    m.insert("middle_01_l", SkinJoint::Joint(JointName::HandMiddle1Left));
    m.insert("middle_02_l", SkinJoint::Joint(JointName::HandMiddle2Left));
    m.insert("middle_03_l", SkinJoint::Joint(JointName::HandMiddle3Left));

    m.insert("ring_01_l", SkinJoint::Joint(JointName::HandRing1Left));
    m.insert("ring_02_l", SkinJoint::Joint(JointName::HandRing2Left));
    m.insert("ring_03_l", SkinJoint::Joint(JointName::HandRing3Left));

    m.insert("pinky_01_l", SkinJoint::Joint(JointName::HandPinky1Left));
    m.insert("pinky_02_l", SkinJoint::Joint(JointName::HandPinky2Left));
    m.insert("pinky_03_l", SkinJoint::Joint(JointName::HandPinky3Left));

    // Right Fingers
    m.insert("thumb_01_r", SkinJoint::Joint(JointName::HandThumb1Right));
    m.insert("thumb_02_r", SkinJoint::Joint(JointName::HandThumb2Right));
    m.insert("thumb_03_r", SkinJoint::Joint(JointName::HandThumb3Right));

    m.insert("index_01_r", SkinJoint::Joint(JointName::HandIndex1Right));
    m.insert("index_02_r", SkinJoint::Joint(JointName::HandIndex2Right));
    m.insert("index_03_r", SkinJoint::Joint(JointName::HandIndex3Right));

    m.insert("middle_01_r", SkinJoint::Joint(JointName::HandMiddle1Right));
    m.insert("middle_02_r", SkinJoint::Joint(JointName::HandMiddle2Right));
    m.insert("middle_03_r", SkinJoint::Joint(JointName::HandMiddle3Right));

    m.insert("ring_01_r", SkinJoint::Joint(JointName::HandRing1Right));
    m.insert("ring_02_r", SkinJoint::Joint(JointName::HandRing2Right));
    m.insert("ring_03_r", SkinJoint::Joint(JointName::HandRing3Right));

    m.insert("pinky_01_r", SkinJoint::Joint(JointName::HandPinky1Right));
    m.insert("pinky_02_r", SkinJoint::Joint(JointName::HandPinky2Right));
    m.insert("pinky_03_r", SkinJoint::Joint(JointName::HandPinky3Right));

    m
});

impl SkinJoint {
    pub fn resolve_joint_name(s: &str) -> Option<SkinJoint> {
        QUATERNIUS_JOINT_ALIASES
            .get(s)
            .copied()
            .or_else(|| JointName::from_str(s).ok().map(SkinJoint::Joint))
    }

    pub fn resolve_joint_and_deform(s: &str) -> Option<SkinJoint> {
        JointName::from_str(s)
            .ok()
            .map(SkinJoint::Joint)
            .or_else(|| DeformName::from_str(s).ok().map(SkinJoint::Deform))
    }

    pub fn from_str_non_bento(s: &str) -> Option<SkinJoint> {
        BVH_JOINT_ALIASES
            .get(s)
            .copied()
            .or_else(|| JointName::from_str(s).ok().map(SkinJoint::Joint))
    }
}

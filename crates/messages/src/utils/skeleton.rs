use glam::Mat4;
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use strum_macros::{Display, EnumString};
use uuid::Uuid;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
/// Definitions for the avatar's skeleton  
pub struct Skeleton {
    /// The joints that make up the skeleton
    pub joints: IndexMap<JointName, Joint>,
    /// The root bone
    /// this can contain many joints, if the object has a partial skeleton.
    pub root: Vec<JointName>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
/// Defines the joint that makes up the skeleton
pub struct Joint {
    /// The name of the joint
    pub name: JointName,
    /// The transforms applied to the joint
    pub transforms: Vec<Transform>,
    /// The transforms applied to the joint based on its parent and child relationship.
    pub local_transforms: Vec<Transform>,
    /// The position in the skeleton's vec of the joint's children
    pub children: Vec<JointName>,
    /// The position in the skeleton's vec of the joint's parent
    pub parent: Option<JointName>,
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

#[allow(missing_docs)]
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
pub enum JointName {
    // Root
    #[strum(serialize = "BUTT")]
    Butt,
    #[strum(serialize = "PELVIS")]
    PelvisLegacy,
    #[default]
    #[strum(serialize = "mPelvis")]
    Pelvis,

    // Spine
    #[strum(serialize = "mSpine1")]
    Spine1,
    #[strum(serialize = "mSpine2")]
    Spine2,
    #[strum(serialize = "mSpine3")]
    Spine3,
    #[strum(serialize = "mSpine4")]
    Spine4,
    #[strum(serialize = "LOWER_BACK")]
    LowerBack,
    #[strum(serialize = "BELLY")]
    Belly,
    #[strum(serialize = "UPPER_BACK")]
    UpperBack,
    #[strum(serialize = "CHEST")]
    ChestLegacy,
    #[strum(serialize = "mChest")]
    Chest,
    #[strum(serialize = "mTorso")]
    Torso,
    #[strum(serialize = "mGroin")]
    Groin,

    #[strum(serialize = "LEFT_HANDLE")]
    HandleLeft,
    #[strum(serialize = "RIGHT_HANDLE")]
    HandleRight,
    // Neck & Head
    #[strum(serialize = "NECK")]
    NeckLegacy,
    #[strum(serialize = "HEAD")]
    HeadLegacy,
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

    // Shoulders
    #[strum(serialize = "L_CLAVICLE")]
    ClavicleLeft,
    #[strum(serialize = "R_CLAVICLE")]
    ClavicleRight,
    #[strum(serialize = "SHOULDER_LEFT")]
    ShoulderLeftLegacy,
    #[strum(serialize = "SHOULDER_RIGHT")]
    ShoulderRightLegacy,
    #[strum(serialize = "mShoulderLeft")]
    ShoulderLeft,
    #[strum(serialize = "mShoulderRight")]
    ShoulderRight,
    #[strum(serialize = "mCollarLeft")]
    CollarLeft,
    #[strum(serialize = "mCollarRight")]
    CollarRight,

    // Arms
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

    // Hands and fingers
    #[strum(serialize = "L_HAND")]
    HandLeftLegacy,
    #[strum(serialize = "R_HAND")]
    HandRightLegacy,
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

    // Legs
    #[strum(serialize = "L_UPPER_LEG")]
    UpperLegLeft,
    #[strum(serialize = "L_LOWER_LEG")]
    LowerLegLeft,
    #[strum(serialize = "R_UPPER_LEG")]
    UpperLegRight,
    #[strum(serialize = "R_LOWER_LEG")]
    LowerLegRight,
    #[strum(serialize = "HIP_LEFT")]
    HipLeftLegacy,
    #[strum(serialize = "HIP_RIGHT")]
    HipRightLegacy,
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
    #[strum(serialize = "KNEE_LEFT")]
    UpperKneeLeft,
    #[strum(serialize = "KNEE_RIGHT")]
    UpperKneeRight,

    // Feet
    #[strum(serialize = "ANKLE_LEFT")]
    AnkleLeftLegacy,
    #[strum(serialize = "ANKLE_RIGHT")]
    AnkleRightLegacy,
    #[strum(serialize = "mAnkleLeft")]
    AnkleLeft,
    #[strum(serialize = "mAnkleRight")]
    AnkleRight,
    #[strum(serialize = "L_FOOT")]
    FootLeftLegacy,
    #[strum(serialize = "R_FOOT")]
    FootRightLegacy2,
    #[strum(serialize = "FOOT_LEFT")]
    FootLeftLegacy2,
    #[strum(serialize = "FOOT_RIGHT")]
    FootRightLegacy,
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

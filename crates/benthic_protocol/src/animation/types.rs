use crate::avatar::skeleton::{Skeleton, SkinJoint};
use glam::{Quat, Vec3};
use serde::{Deserialize, Serialize};

/// Defines an animation clip.
/// Bind skeleton is the skeleton this animaion was authored for, which is used in retargeting.
/// Joints contains the transforms at each keyframe.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnimationClip {
    /// Bind skeleton the animation was authored for
    pub bind_skeleton: Skeleton,
    /// List of animation keyframes for each joint.
    /// TODO: this should probably be renamed or reconsidered.
    /// having it be a vector of joints is weird. Maybe a btree set?
    pub joints: Vec<JointAnimation>,
}

/// Defines per-joint animations.
/// Contains translations, rotations and scales for each joint.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JointAnimation {
    /// Joint that this animation targets
    pub joint: SkinJoint,
    /// List of all translation keyframes
    pub translations: Vec<Keyframe<Vec3>>,
    /// List of all rotation keyframes
    pub rotations: Vec<Keyframe<Quat>>,
    /// List of all scale keyframes
    pub scales: Vec<Keyframe<Vec3>>,
}

/// Defines keyframes.
/// Generic T can be either a Vec3 or a Quat, used to represent translation, scale or rotation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Keyframe<T> {
    /// Time when this change happens
    pub time: f32,
    /// Value of the change
    pub value: T,
}

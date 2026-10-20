use crate::avatar::skeleton::{Skeleton, SkinJoint};
use glam::{Quat, Vec3};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnimationClip {
    pub bind_skeleton: Skeleton,
    pub joints: Vec<JointAnimation>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JointAnimation {
    pub joint: SkinJoint,
    pub translations: Vec<Keyframe<Vec3>>,
    pub rotations: Vec<Keyframe<Quat>>,
    pub scales: Vec<Keyframe<Vec3>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Keyframe<T> {
    pub time: f32,
    pub value: T,
}

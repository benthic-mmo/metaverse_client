use crate::skeleton::{JointName, Skeleton};
use glam::{Quat, Vec3};
use serde::{Deserialize, Serialize};
use strum_macros::{Display, EnumString};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnimationClip {
    pub bind_skeleton: Skeleton,
    pub joints: Vec<JointAnimation>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JointAnimation {
    pub joint: JointName,
    pub translations: Vec<Keyframe<Vec3>>,
    pub rotations: Vec<Keyframe<Quat>>,
    pub scales: Vec<Keyframe<Vec3>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Keyframe<T> {
    pub time: f32,
    pub value: T,
}

use uuid::Uuid;

macro_rules! define_animations {
    (
        $( $name:ident => $uuid:expr ),* $(,)?
    ) => {
        #[derive(Debug, Copy, Clone, Display, EnumString, PartialEq, Eq, Hash)]
        pub enum DefaultAnimation {
            $( $name ),*
        }

        impl DefaultAnimation {
            pub fn uuid(&self) -> Uuid {
                match self {
                    $(
                        DefaultAnimation::$name => Uuid::parse_str($uuid).unwrap(),
                    )*
                }
            }

            pub fn from_uuid(uuid: &Uuid) -> Option<Self> {
                $(
                    if Uuid::parse_str($uuid).unwrap() == *uuid {
                        return Some(DefaultAnimation::$name);
                    }
                )*
                None
            }
            pub fn from_string(name: &str) -> Option<Self> {
                name.parse().ok()
            }
        }
    };
}

define_animations! {
    Stand => "2408fe9e-df1d-1d7d-f4ff-1384fa7b350f",
    Dance => "2408fe9e-df1d-1d7d-f4ff-1384fa7b350a"
}

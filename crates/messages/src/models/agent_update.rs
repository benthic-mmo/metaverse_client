use std::{collections::HashMap, sync::Arc};
use tokio::sync::oneshot::Sender;
use std::sync::Mutex;

use nalgebra::Quaternion;
use uuid::Uuid;

use super::{
    client_update_data::ClientUpdateData,
    header::{Header, PacketFrequency},
    packet::{MessageType, Packet, PacketData},
};

const AGENT_STATE_TYPING: u8 = 0x04; // 00000100 in binary
const AGENT_STATE_EDITING: u8 = 0x10; // 00010000 in binary

const AGENT_CONTROL_AT_POS: u32 = 0x00000001;
const AGENT_CONTROL_AT_NEG: u32 = 0x00000002;
const AGENT_CONTROL_LEFT_POS: u32 = 0x00000004;
const AGENT_CONTROL_LEFT_NEG: u32 = 0x00000008;
const AGENT_CONTROL_UP_POS: u32 = 0x00000010;
const AGENT_CONTROL_UP_NEG: u32 = 0x00000020;
const AGENT_CONTROL_PITCH_POS: u32 = 0x00000040;
const AGENT_CONTROL_PITCH_NEG: u32 = 0x00000080;
const AGENT_CONTROL_YAW_POS: u32 = 0x00000100;
const AGENT_CONTROL_YAW_NEG: u32 = 0x00000200;
const AGENT_CONTROL_FAST_AT: u32 = 0x00000400;
const AGENT_CONTROL_FAST_LEFT: u32 = 0x00000800;
const AGENT_CONTROL_FAST_UP: u32 = 0x00001000;
const AGENT_CONTROL_FLY: u32 = 0x00002000;
const AGENT_CONTROL_STOP: u32 = 0x00004000;
const AGENT_CONTROL_FINISH_ANIM: u32 = 0x00008000;
const AGENT_CONTROL_STAND_UP: u32 = 0x00010000;
const AGENT_CONTROL_SIT_ON_GROUND: u32 = 0x00020000;
const AGENT_CONTROL_MOUSELOOK: u32 = 0x00040000;
const AGENT_CONTROL_NUDGE_AT_POS: u32 = 0x00080000;
const AGENT_CONTROL_NUDGE_AT_NEG: u32 = 0x00100000;
const AGENT_CONTROL_NUDGE_LEFT_POS: u32 = 0x00200000;
const AGENT_CONTROL_NUDGE_LEFT_NEG: u32 = 0x00400000;
const AGENT_CONTROL_NUDGE_UP_POS: u32 = 0x00800000;
const AGENT_CONTROL_NUDGE_UP_NEG: u32 = 0x01000000;
const AGENT_CONTROL_TURN_LEFT: u32 = 0x02000000;
const AGENT_CONTROL_TURN_RIGHT: u32 = 0x04000000;
const AGENT_CONTROL_AWAY: u32 = 0x08000000;
const AGENT_CONTROL_LBUTTON_DOWN: u32 = 0x10000000;
const AGENT_CONTROL_LBUTTON_UP: u32 = 0x20000000;
const AGENT_CONTROL_ML_LBUTTON_DOWN: u32 = 0x40000000;
const AGENT_CONTROL_ML_LBUTTON_UP: u32 = 0x80000000;

const AU_FLAGS_NONE: u8 = 0x00;
const AU_FLAGS_HIDETITLE: u8 = 0x01;

impl Packet {
    pub fn new_agent_update(agent_update: AgentUpdate) -> Self {
        Packet {
            header: Header {
                id: 4,
                frequency: PacketFrequency::High,
                reliable: false,
                sequence_number: 1,
                appended_acks: false,
                zerocoded: false,
                resent: false,
                ack_list: None,
                size: None,
            },
            body: Arc::new(agent_update),
        }
    }
}

#[derive(Debug)]
pub struct Vector3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

#[derive(Debug)]
pub struct State {
    pub typing: bool,
    pub editing: bool,
}
impl State {
    pub fn from_bytes(bits: u8) -> Self {
        Self {
            typing: bits & AGENT_STATE_TYPING != 0,
            editing: bits & AGENT_STATE_EDITING != 0,
        }
    }
    pub fn to_bytes(&self) -> u8 {
        let mut bits = 0u8;
        if self.typing {
            bits |= AGENT_STATE_TYPING;
        }
        if self.editing {
            bits |= AGENT_STATE_EDITING;
        }
        bits
    }
}

#[derive(Debug)]
pub struct ControlFlags {
    pub at_pos: bool,
    pub at_neg: bool,
    pub left_pos: bool,
    pub left_neg: bool,
    pub up_pos: bool,
    pub up_neg: bool,
    pub pitch_pos: bool,
    pub pitch_neg: bool,
    pub yaw_pos: bool,
    pub yaw_neg: bool,
    pub fast_at: bool,
    pub fast_left: bool,
    pub fast_up: bool,
    pub fly: bool,
    pub stop: bool,
    pub finish_anim: bool,
    pub stand_up: bool,
    pub sit_on_ground: bool,
    pub mouselook: bool,
    pub nudge_at_pos: bool,
    pub nudge_at_neg: bool,
    pub nudge_left_pos: bool,
    pub nudge_left_neg: bool,
    pub nudge_up_pos: bool,
    pub nudge_up_neg: bool,
    pub turn_left: bool,
    pub turn_right: bool,
    pub away: bool,
    pub l_button_down: bool,
    pub l_button_up: bool,
    pub ml_button_down: bool,
    pub ml_button_up: bool,
}
impl ControlFlags {
    pub fn from_bytes(bits: u32) -> Self {
        Self {
            at_pos: bits & AGENT_CONTROL_AT_POS != 0,
            at_neg: bits & AGENT_CONTROL_AT_NEG != 0,
            left_pos: bits & AGENT_CONTROL_LEFT_POS != 0,
            left_neg: bits & AGENT_CONTROL_LEFT_NEG != 0,
            up_pos: bits & AGENT_CONTROL_UP_POS != 0,
            up_neg: bits & AGENT_CONTROL_UP_NEG != 0,
            pitch_pos: bits & AGENT_CONTROL_PITCH_POS != 0,
            pitch_neg: bits & AGENT_CONTROL_PITCH_NEG != 0,
            yaw_pos: bits & AGENT_CONTROL_YAW_POS != 0,
            yaw_neg: bits & AGENT_CONTROL_YAW_NEG != 0,
            fast_at: bits & AGENT_CONTROL_FAST_AT != 0,
            fast_left: bits & AGENT_CONTROL_FAST_LEFT != 0,
            fast_up: bits & AGENT_CONTROL_FAST_UP != 0,
            fly: bits & AGENT_CONTROL_FLY != 0,
            stop: bits & AGENT_CONTROL_STOP != 0,
            finish_anim: bits & AGENT_CONTROL_FINISH_ANIM != 0,
            stand_up: bits & AGENT_CONTROL_STAND_UP != 0,
            sit_on_ground: bits & AGENT_CONTROL_SIT_ON_GROUND != 0,
            mouselook: bits & AGENT_CONTROL_MOUSELOOK != 0,
            nudge_at_pos: bits & AGENT_CONTROL_NUDGE_AT_POS != 0,
            nudge_at_neg: bits & AGENT_CONTROL_NUDGE_AT_NEG != 0,
            nudge_left_pos: bits & AGENT_CONTROL_NUDGE_LEFT_POS != 0,
            nudge_left_neg: bits & AGENT_CONTROL_NUDGE_LEFT_NEG != 0,
            nudge_up_pos: bits & AGENT_CONTROL_NUDGE_UP_POS != 0,
            nudge_up_neg: bits & AGENT_CONTROL_NUDGE_UP_NEG != 0,
            turn_left: bits & AGENT_CONTROL_TURN_LEFT != 0,
            turn_right: bits & AGENT_CONTROL_TURN_RIGHT != 0,
            away: bits & AGENT_CONTROL_AWAY != 0,
            l_button_down: bits & AGENT_CONTROL_LBUTTON_DOWN != 0,
            l_button_up: bits & AGENT_CONTROL_LBUTTON_UP != 0,
            ml_button_down: bits & AGENT_CONTROL_ML_LBUTTON_DOWN != 0,
            ml_button_up: bits & AGENT_CONTROL_ML_LBUTTON_UP != 0,
        }
    }
    pub fn to_bytes(&self) -> [u8; 4] {
        let mut bits = 0u32;
        if self.at_pos {
            bits |= AGENT_CONTROL_AT_POS;
        }
        if self.at_neg {
            bits |= AGENT_CONTROL_AT_NEG;
        }
        if self.left_pos {
            bits |= AGENT_CONTROL_LEFT_POS;
        }
        if self.left_neg {
            bits |= AGENT_CONTROL_LEFT_NEG;
        }
        if self.up_pos {
            bits |= AGENT_CONTROL_UP_POS;
        }
        if self.up_neg {
            bits |= AGENT_CONTROL_UP_NEG;
        }
        if self.pitch_pos {
            bits |= AGENT_CONTROL_PITCH_POS;
        }
        if self.pitch_neg {
            bits |= AGENT_CONTROL_PITCH_NEG;
        }
        if self.yaw_pos {
            bits |= AGENT_CONTROL_YAW_POS;
        }
        if self.yaw_neg {
            bits |= AGENT_CONTROL_YAW_NEG;
        }
        if self.fast_at {
            bits |= AGENT_CONTROL_FAST_AT;
        }
        if self.fast_left {
            bits |= AGENT_CONTROL_FAST_LEFT;
        }
        if self.fast_up {
            bits |= AGENT_CONTROL_FAST_UP;
        }
        if self.fly {
            bits |= AGENT_CONTROL_FLY;
        }
        if self.stop {
            bits |= AGENT_CONTROL_STOP;
        }
        if self.finish_anim {
            bits |= AGENT_CONTROL_FINISH_ANIM;
        }
        if self.stand_up {
            bits |= AGENT_CONTROL_STAND_UP;
        }
        if self.sit_on_ground {
            bits |= AGENT_CONTROL_SIT_ON_GROUND;
        }
        if self.mouselook {
            bits |= AGENT_CONTROL_MOUSELOOK;
        }
        if self.nudge_at_pos {
            bits |= AGENT_CONTROL_NUDGE_AT_POS;
        }
        if self.nudge_at_neg {
            bits |= AGENT_CONTROL_NUDGE_AT_NEG;
        }
        if self.nudge_left_pos {
            bits |= AGENT_CONTROL_NUDGE_LEFT_POS;
        }
        if self.nudge_left_neg {
            bits |= AGENT_CONTROL_NUDGE_LEFT_NEG;
        }
        if self.nudge_up_pos {
            bits |= AGENT_CONTROL_NUDGE_UP_POS;
        }
        if self.nudge_up_neg {
            bits |= AGENT_CONTROL_NUDGE_UP_NEG;
        }
        if self.turn_left {
            bits |= AGENT_CONTROL_TURN_LEFT;
        }
        if self.turn_right {
            bits |= AGENT_CONTROL_TURN_RIGHT;
        }
        if self.away {
            bits |= AGENT_CONTROL_AWAY;
        }
        if self.l_button_down {
            bits |= AGENT_CONTROL_LBUTTON_DOWN;
        }
        if self.l_button_up {
            bits |= AGENT_CONTROL_LBUTTON_UP;
        }
        if self.ml_button_down {
            bits |= AGENT_CONTROL_ML_LBUTTON_DOWN;
        }
        if self.ml_button_up {
            bits |= AGENT_CONTROL_ML_LBUTTON_UP;
        }
        bits.to_le_bytes()
    }
}

#[derive(Debug)]
pub struct Flags {
    pub none: bool,
    pub hide_title: bool,
}
impl Flags {
    pub fn from_bytes(bits: u8) -> Self {
        Self {
            none: bits & AU_FLAGS_NONE != 0,
            hide_title: bits & AU_FLAGS_HIDETITLE != 0,
        }
    }
    pub fn to_bytes(&self) -> u8 {
        let mut bits = 0u8;
        if self.none {
            bits |= AU_FLAGS_NONE;
        }
        if self.hide_title {
            bits |= AU_FLAGS_HIDETITLE;
        }
        bits
    }
}

#[derive(Debug)]
pub struct AgentUpdate {
    pub agent_id: Uuid,
    pub session_id: Uuid,
    pub body_rotation: Quaternion<f32>,
    pub head_rotation: Quaternion<f32>,
    pub state: State,
    pub camera_center: Vector3,
    pub camera_at_axis: Vector3,
    pub camera_left_axis: Vector3,
    pub camera_up_axis: Vector3,
    pub far: f32,
    pub control_flags: ControlFlags,
    pub flags: Flags,
}

pub trait ToFromBytes {
    fn from_bytes(bytes: &[u8]) -> Self;
    fn to_bytes(&self) -> [u8; 16];
}

impl ToFromBytes for Quaternion<f32> {
    fn from_bytes(bytes: &[u8]) -> Self {
        let x = f32::from_le_bytes(bytes[0..4].try_into().unwrap());
        let y = f32::from_le_bytes(bytes[4..8].try_into().unwrap());
        let z = f32::from_le_bytes(bytes[8..12].try_into().unwrap());
        let w = f32::from_le_bytes(bytes[12..16].try_into().unwrap());
        Quaternion::new(w, x, y, z)
    }
    fn to_bytes(&self) -> [u8; 16] {
        let mut bytes = [0u8; 16];
        bytes[0..4].copy_from_slice(&self.w.to_le_bytes());
        bytes[4..8].copy_from_slice(&self.i.to_le_bytes());
        bytes[8..12].copy_from_slice(&self.j.to_le_bytes());
        bytes[12..16].copy_from_slice(&self.k.to_le_bytes());
        bytes
    }
}

impl PacketData for AgentUpdate {
    fn from_bytes(bytes: &[u8]) -> std::io::Result<Self> {
        let agent_id = Uuid::from_slice(&bytes[0..16]).unwrap();
        let session_id = Uuid::from_slice(&bytes[16..32]).unwrap();
        let body_rotation = Quaternion::from_bytes(&bytes[32..48]);
        let head_rotation = Quaternion::from_bytes(&bytes[48..64]);

        let state = State::from_bytes(bytes[64]);
        let camera_center = Vector3 {
            x: f32::from_le_bytes(bytes[65..69].try_into().unwrap()),
            y: f32::from_le_bytes(bytes[69..73].try_into().unwrap()),
            z: f32::from_le_bytes(bytes[73..77].try_into().unwrap()),
        };
        let camera_at_axis = Vector3 {
            x: f32::from_le_bytes(bytes[77..81].try_into().unwrap()),
            y: f32::from_le_bytes(bytes[81..85].try_into().unwrap()),
            z: f32::from_le_bytes(bytes[85..89].try_into().unwrap()),
        };
        let camera_left_axis = Vector3 {
            x: f32::from_le_bytes(bytes[89..93].try_into().unwrap()),
            y: f32::from_le_bytes(bytes[93..97].try_into().unwrap()),
            z: f32::from_le_bytes(bytes[97..101].try_into().unwrap()),
        };
        let camera_up_axis = Vector3 {
            x: f32::from_le_bytes(bytes[101..105].try_into().unwrap()),
            y: f32::from_le_bytes(bytes[105..109].try_into().unwrap()),
            z: f32::from_le_bytes(bytes[109..113].try_into().unwrap()),
        };
        let far = f32::from_le_bytes(bytes[113..117].try_into().unwrap());
        let control_flags =
            ControlFlags::from_bytes(u32::from_le_bytes(bytes[117..121].try_into().unwrap()));
        let flags = Flags::from_bytes(bytes[121]);
        Ok(Self {
            agent_id,
            session_id,
            body_rotation,
            head_rotation,
            state,
            camera_center,
            camera_at_axis,
            camera_left_axis,
            camera_up_axis,
            far,
            control_flags,
            flags,
        })
    }

    fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(121); // Total byte length

        // Serialize UUIDs
        bytes.extend_from_slice(self.agent_id.as_bytes());
        bytes.extend_from_slice(self.session_id.as_bytes());

        // Serialize Quaternions
        bytes.extend_from_slice(&self.body_rotation.to_bytes());
        bytes.extend_from_slice(&self.head_rotation.to_bytes());

        // Serialize State
        bytes.push(self.state.to_bytes());

        // Serialize Vector3s
        bytes.extend_from_slice(&self.camera_center.x.to_le_bytes());
        bytes.extend_from_slice(&self.camera_center.y.to_le_bytes());
        bytes.extend_from_slice(&self.camera_center.z.to_le_bytes());
        bytes.extend_from_slice(&self.camera_at_axis.x.to_le_bytes());
        bytes.extend_from_slice(&self.camera_at_axis.y.to_le_bytes());
        bytes.extend_from_slice(&self.camera_at_axis.z.to_le_bytes());
        bytes.extend_from_slice(&self.camera_left_axis.x.to_le_bytes());
        bytes.extend_from_slice(&self.camera_left_axis.y.to_le_bytes());
        bytes.extend_from_slice(&self.camera_left_axis.z.to_le_bytes());
        bytes.extend_from_slice(&self.camera_up_axis.x.to_le_bytes());
        bytes.extend_from_slice(&self.camera_up_axis.y.to_le_bytes());
        bytes.extend_from_slice(&self.camera_up_axis.z.to_le_bytes());

        bytes.extend_from_slice(&self.far.to_le_bytes());

        bytes.extend_from_slice(&self.control_flags.to_bytes());

        bytes.push(self.flags.to_bytes());

        bytes
    }

    fn on_receive(
        &self,
        _: Arc<Mutex<HashMap<u32, Sender<()>>>>,
        _: Arc<Mutex<Vec<ClientUpdateData>>>,
    ) -> futures::future::BoxFuture<'static, ()> {
        Box::pin(async move {
            // Implement the actual logic here later
            println!("on_receive is not yet implemented.");
        })
    }

    fn message_type(&self) -> super::packet::MessageType {
        MessageType::Outgoing
    }
}

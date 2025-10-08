use crate::packet::{
    errors::PacketError,
    header::{Header, PacketFrequency},
    packet::{Packet, PacketData},
    packet_types::PacketType,
};
use glam::{Quat, Vec3};
use uuid::Uuid;

/// Flag for setting typing state
const AGENT_STATE_TYPING: u8 = 0x04; // 00000100 in binary
/// flag for setting editing state
const AGENT_STATE_EDITING: u8 = 0x10; // 00010000 in binary

/// undocumented
const AGENT_CONTROL_AT_POS: u32 = 0x00000001;
/// undocumented
const AGENT_CONTROL_AT_NEG: u32 = 0x00000002;
/// undocumented
const AGENT_CONTROL_LEFT_POS: u32 = 0x00000004;
/// undocumented
const AGENT_CONTROL_LEFT_NEG: u32 = 0x00000008;
/// undocumented
const AGENT_CONTROL_UP_POS: u32 = 0x00000010;
/// undocumented
const AGENT_CONTROL_UP_NEG: u32 = 0x00000020;
/// undocumented
const AGENT_CONTROL_PITCH_POS: u32 = 0x00000040;
/// undocumented
const AGENT_CONTROL_PITCH_NEG: u32 = 0x00000080;
/// undocumented
const AGENT_CONTROL_YAW_POS: u32 = 0x00000100;
/// undocumented
const AGENT_CONTROL_YAW_NEG: u32 = 0x00000200;
/// undocumented
const AGENT_CONTROL_FAST_AT: u32 = 0x00000400;
/// undocumented
const AGENT_CONTROL_FAST_LEFT: u32 = 0x00000800;
/// undocumented
const AGENT_CONTROL_FAST_UP: u32 = 0x00001000;
/// Set while flying
const AGENT_CONTROL_FLY: u32 = 0x00002000;
/// Soft freezes the avatar
const AGENT_CONTROL_STOP: u32 = 0x00004000;
/// undocumented
const AGENT_CONTROL_FINISH_ANIM: u32 = 0x00008000;
/// Set once to stand up from sitting
const AGENT_CONTROL_STAND_UP: u32 = 0x00010000;
/// set while sitting on the ground
const AGENT_CONTROL_SIT_ON_GROUND: u32 = 0x00020000;
/// Set while in mouse look
const AGENT_CONTROL_MOUSELOOK: u32 = 0x00040000;
/// undocumented
const AGENT_CONTROL_NUDGE_AT_POS: u32 = 0x00080000;
/// undocumented
const AGENT_CONTROL_NUDGE_AT_NEG: u32 = 0x00100000;
/// undocumented
const AGENT_CONTROL_NUDGE_LEFT_POS: u32 = 0x00200000;
/// undocumented
const AGENT_CONTROL_NUDGE_LEFT_NEG: u32 = 0x00400000;
/// undocumented
const AGENT_CONTROL_NUDGE_UP_POS: u32 = 0x00800000;
/// undocumented
const AGENT_CONTROL_NUDGE_UP_NEG: u32 = 0x01000000;
/// undocumented
const AGENT_CONTROL_TURN_LEFT: u32 = 0x02000000;
/// undocumented
const AGENT_CONTROL_TURN_RIGHT: u32 = 0x04000000;
/// Set while away
const AGENT_CONTROL_AWAY: u32 = 0x08000000;
/// undocumented
const AGENT_CONTROL_LBUTTON_DOWN: u32 = 0x10000000;
/// undocumented
const AGENT_CONTROL_LBUTTON_UP: u32 = 0x20000000;
/// undocumented
const AGENT_CONTROL_ML_LBUTTON_DOWN: u32 = 0x40000000;
/// undocumented
const AGENT_CONTROL_ML_LBUTTON_UP: u32 = 0x80000000;

const AU_FLAGS_NONE: u8 = 0x00;
const AU_FLAGS_HIDETITLE: u8 = 0x01;

impl Packet {
    /// Creates a new agent update packet
    /// agent_update: the agent update object to add to the packet
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
            body: PacketType::AgentUpdate(Box::new(agent_update)),
        }
    }
}

#[derive(Debug, Clone)]
/// User states. Currently only typing and editing.
/// used to display typing animations, and edit mode indicators.
pub struct State {
    /// Is the user typing
    pub typing: bool,
    /// Is the user in edit mode
    pub editing: bool,
}
impl State {
    fn default() -> Self {
        Self {
            typing: false,
            editing: false,
        }
    }
    /// Converts the bits containing the state information to booleans using a bitwise and
    pub fn from_bytes(bits: u8) -> Self {
        Self {
            typing: bits & AGENT_STATE_TYPING != 0,
            editing: bits & AGENT_STATE_EDITING != 0,
        }
    }

    /// Converts the boolean state information to bits using a bitwise or
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

#[derive(Debug, Clone)]
/// Defines the actions an Agent Update packet can take
pub struct ControlFlags {
    /// undocumented
    pub at_pos: bool,
    /// undocumented
    pub at_neg: bool,
    /// undocumented
    pub left_pos: bool,
    /// undocumented
    pub left_neg: bool,
    /// undocumented
    pub up_pos: bool,
    /// undocumented
    pub up_neg: bool,
    /// undocumented
    pub pitch_pos: bool,
    /// undocumented
    pub pitch_neg: bool,
    /// undocumented
    pub yaw_pos: bool,
    /// undocumented
    pub yaw_neg: bool,
    /// undocumented
    pub fast_at: bool,
    /// undocumented
    pub fast_left: bool,
    /// undocumented
    pub fast_up: bool,
    /// Set while flying
    pub fly: bool,
    /// Soft freezes the avatar
    pub stop: bool,
    /// undocumented
    pub finish_anim: bool,
    /// Sent once to stand up
    pub stand_up: bool,
    /// Set while sitting on the ground
    pub sit_on_ground: bool,
    /// set while in mouselook
    pub mouselook: bool,
    /// undocumented
    pub nudge_at_pos: bool,
    /// undocumented
    pub nudge_at_neg: bool,
    /// undocumented
    pub nudge_left_pos: bool,
    /// undocumented
    pub nudge_left_neg: bool,
    /// undocumented
    pub nudge_up_pos: bool,
    /// undocumented
    pub nudge_up_neg: bool,
    /// undocumented
    pub turn_left: bool,
    /// undocumented
    pub turn_right: bool,
    /// Set while away
    pub away: bool,
    /// undocumented
    pub l_button_down: bool,
    /// undocumented
    pub l_button_up: bool,
    /// undocumented
    pub ml_button_down: bool,
    /// undocumented
    pub ml_button_up: bool,
}

impl ControlFlags {
    fn default() -> Self {
        Self {
            at_pos: false,
            at_neg: false,
            left_pos: false,
            left_neg: false,
            up_pos: false,
            up_neg: false,
            pitch_pos: false,
            pitch_neg: false,
            yaw_pos: false,
            yaw_neg: false,
            fast_at: false,
            fast_left: false,
            fast_up: false,
            fly: false,
            stop: false,
            finish_anim: false,
            stand_up: false,
            sit_on_ground: false,
            mouselook: false,
            nudge_at_pos: false,
            nudge_at_neg: false,
            nudge_left_pos: false,
            nudge_left_neg: false,
            nudge_up_pos: false,
            nudge_up_neg: false,
            turn_left: false,
            turn_right: false,
            away: false,
            l_button_down: false,
            l_button_up: false,
            ml_button_down: false,
            ml_button_up: false,
        }
    }
    /// Converts control flags bits to bools using bitwise ands
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
    /// Converts boolean controlflags to bits using bitwise ors
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

#[derive(Debug, Clone)]
/// Struct for hide title flags.
pub struct Flags {
    /// no flags at all
    pub none: bool,
    /// Used to hide group title in nametag. Hides title locally and for everyone else.
    pub hide_title: bool,
}
#[allow(clippy::bad_bit_mask)]
impl Flags {
    fn default() -> Self {
        Self {
            none: false,
            hide_title: false,
        }
    }
    /// Converts from bits to boolean using bitwise and
    pub fn from_bytes(bits: u8) -> Self {
        Self {
            none: bits & AU_FLAGS_NONE != 0,
            hide_title: bits & AU_FLAGS_HIDETITLE != 0,
        }
    }
    /// Converts from boolean to bits using bitwise or
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
#[derive(Debug, Clone)]
/// AgentUpdate struct. Regularly sent to the viewer in order to keep the viewer updated about the
/// user's location and movement.
pub struct AgentUpdate {
    /// The id of the agent, sent to the client from the server after login
    pub agent_id: Uuid,
    /// the id of the session, sent to the client from the server after login
    pub session_id: Uuid,
    /// rotation of the user's body
    pub body_rotation: Quat,
    /// rotation of the user's head
    pub head_rotation: Quat,
    /// typing or editing state
    pub state: State,
    /// location of the camera in region local coordinates
    pub camera_center: Vec3,
    /// x rotational axis of the camera
    pub camera_at_axis: Vec3,
    /// y rotational axis of the camera
    pub camera_left_axis: Vec3,
    /// z rotational axis of the camera
    pub camera_up_axis: Vec3,
    /// the distance the viewer can see in meters
    pub far: f32,
    /// info about the actions the agent is taking this frame
    pub control_flags: ControlFlags,
    /// wether or not to hide title
    pub flags: Flags,
}
impl Default for AgentUpdate {
    fn default() -> Self {
        Self {
            agent_id: Uuid::nil(),
            session_id: Uuid::nil(),
            body_rotation: Quat::IDENTITY,
            head_rotation: Quat::IDENTITY,
            state: State::default(),
            camera_center: Vec3::ZERO,
            camera_at_axis: Vec3::Z,
            camera_left_axis: Vec3::X,
            camera_up_axis: Vec3::Y,
            far: 200.0,
            control_flags: ControlFlags::default(),
            flags: Flags::default(),
        }
    }
}

/// Traits tha allows converting quaternions to and from bytes compatable with the protocol's
/// layout.
pub trait ToFromBytes {
    /// parse from bytes
    fn from_bytes(bytes: &[u8]) -> Self;
    /// convert to bytes
    fn to_bytes(&self) -> [u8; 16];
}

/// local type for quaternion handling
/// TODO: delete this.
pub struct QuatBytes(pub Quat);

impl QuatBytes {
    /// convert bytes to a quaternion
    pub fn from_bytes(bytes: &[u8]) -> Result<Quat, PacketError> {
        let x = f32::from_le_bytes(bytes[0..4].try_into()?);
        let y = f32::from_le_bytes(bytes[4..8].try_into()?);
        let z = f32::from_le_bytes(bytes[8..12].try_into()?);
        let w = f32::from_le_bytes(bytes[12..16].try_into()?);
        Ok(Quat::from_xyzw(x, y, z, w))
    }
    /// convert quaternions to bytes
    fn to_bytes(&self) -> [u8; 16] {
        let mut bytes = [0u8; 16];
        bytes[0..4].copy_from_slice(&self.0.w.to_le_bytes());
        bytes[4..8].copy_from_slice(&self.0.x.to_le_bytes());
        bytes[8..12].copy_from_slice(&self.0.y.to_le_bytes());
        bytes[12..16].copy_from_slice(&self.0.z.to_le_bytes());
        bytes
    }
}

impl PacketData for AgentUpdate {
    fn from_bytes(bytes: &[u8]) -> Result<Self, PacketError> {
        // THIS DOES NOT WORK AT ALL
        // THIS WILL CRASH AND BREAK YOUR SHIT
        let agent_id = Uuid::from_slice(&bytes[0..16])?;
        let session_id = Uuid::from_slice(&bytes[16..32])?;
        let body_rotation = QuatBytes::from_bytes(&bytes[32..48])?;
        let head_rotation = QuatBytes::from_bytes(&bytes[48..64])?;

        let state = State::from_bytes(bytes[64]);
        let camera_center = Vec3 {
            x: f32::from_le_bytes(bytes[65..69].try_into()?),
            y: f32::from_le_bytes(bytes[69..73].try_into()?),
            z: f32::from_le_bytes(bytes[73..77].try_into()?),
        };
        let camera_at_axis = Vec3 {
            x: f32::from_le_bytes(bytes[77..81].try_into()?),
            y: f32::from_le_bytes(bytes[81..85].try_into()?),
            z: f32::from_le_bytes(bytes[85..89].try_into()?),
        };
        let camera_left_axis = Vec3 {
            x: f32::from_le_bytes(bytes[89..93].try_into()?),
            y: f32::from_le_bytes(bytes[93..97].try_into()?),
            z: f32::from_le_bytes(bytes[97..101].try_into()?),
        };
        let camera_up_axis = Vec3 {
            x: f32::from_le_bytes(bytes[101..105].try_into()?),
            y: f32::from_le_bytes(bytes[105..109].try_into()?),
            z: f32::from_le_bytes(bytes[109..113].try_into()?),
        };
        let far = f32::from_le_bytes(bytes[113..117].try_into()?);
        let control_flags =
            ControlFlags::from_bytes(u32::from_le_bytes(bytes[117..121].try_into()?));
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
        bytes.extend_from_slice(&QuatBytes(self.body_rotation).to_bytes());
        bytes.extend_from_slice(&QuatBytes(self.body_rotation).to_bytes());

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
}

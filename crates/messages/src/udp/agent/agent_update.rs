use crate::errors::ParseError;
use crate::packet::{
    header::{Header, PacketFrequency},
    packet::{Packet, PacketData},
    packet_types::PacketType,
};
use benthic_protocol::messages::utils::agent_update_types::{
    ControlFlags, Flags, QuatBytes, State,
};
use glam::{Quat, Vec3};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

impl Packet {
    /// Creates a new agent update packet
    /// agent_update: the agent update object to add to the packet
    pub fn new_agent_update(agent_update: AgentUpdate) -> Self {
        Packet {
            header: Header {
                id: 4,
                frequency: PacketFrequency::High,
                reliable: false,
                zerocoded: false,
                ..Default::default()
            },
            body: PacketType::AgentUpdate(Box::new(agent_update)),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
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

impl PacketData for AgentUpdate {
    fn from_bytes(bytes: &[u8]) -> Result<Self, ParseError> {
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

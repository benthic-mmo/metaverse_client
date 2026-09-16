use glam::{Quat, Vec3};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::messages::{
    ui::ui_messages::UIResponse,
    utils::agent_update_types::{ControlFlags, Flags, State},
};

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

impl UIResponse {
    /// create a new UI message to allow the client to inform the server of agent updates
    pub fn new_agent_update(data: AgentUpdate) -> Self {
        UIResponse::AgentUpdate(data)
    }
}
